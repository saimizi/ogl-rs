#[allow(unused)]
use {
    core::borrow::Borrow,
    error_stack::{Report, Result, ResultExt},
    jlogger_tracing::{
        jdebug, jerror, jinfo, jtrace, jwarn, JloggerBuilder, LevelFilter, LogTimeFormat,
    },
    libogl::error::OglError,
    std::sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    std::{fs::File, os::fd::AsFd},
    std::{thread::sleep, time::Duration},
    wayland_client::{
        backend::{Backend, ObjectId},
        delegate_noop,
        protocol::wl_keyboard::{self, KeyState},
        protocol::{
            wl_buffer, wl_compositor, wl_registry, wl_seat, wl_shm, wl_shm_pool, wl_surface,
        },
        Connection, Dispatch, EventQueue, Proxy, QueueHandle, WEnum,
    },
    wayland_egl::WlEglSurface,
    wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base},
};

#[derive(Default)]
pub struct WaylandEventCb {
    pub key_pressed: Option<Box<dyn Fn(u32)>>,
    pub key_released: Option<Box<dyn Fn(u32)>>,
}

#[derive(Default)]
pub struct WaylandStateInner {
    pub conn: Option<Connection>,
    pub comp: Option<wl_compositor::WlCompositor>,
    pub surface: Option<wl_surface::WlSurface>,
    pub shm: Option<wl_shm::WlShm>,
    pub xdg_base: Option<xdg_wm_base::XdgWmBase>,
    pub xdg_surface: Option<(xdg_surface::XdgSurface, xdg_toplevel::XdgToplevel)>,
    pub configured: bool,
    pub egl_window: Option<WlEglSurface>,
    pub event_cb: Option<WaylandEventCb>,
}

#[derive(Default)]
pub struct WaylandState {
    event_queue: Option<EventQueue<WaylandStateInner>>,
    inner: WaylandStateInner,
}

impl WaylandState {
    pub fn new(event_cb: Option<WaylandEventCb>) -> Result<Self, OglError> {
        let mut ws = WaylandState::default();
        ws.inner.event_cb = event_cb;

        let conn = Connection::connect_to_env().map_err(|e| {
            Report::new(OglError::WaylandError)
                .attach_printable(format!("Failed to connect to wayland server: {e}"))
        })?;

        let mut event_queue = conn.new_event_queue();
        let qh = event_queue.handle();

        let display = conn.display();

        display.get_registry(&qh, ());
        jdebug!("round trip.");
        event_queue.roundtrip(&mut ws.inner).map_err(|e| {
            Report::new(OglError::WaylandError).attach_printable(format!("Failed to dispatch: {e}"))
        })?;

        assert_ne!(ws.inner.comp, None);
        assert_ne!(ws.inner.surface, None);
        assert_ne!(ws.inner.xdg_base, None);
        assert_ne!(ws.inner.xdg_surface, None);
        ws.inner.conn = Some(conn);
        ws.event_queue = Some(event_queue);

        Ok(ws)
    }

    pub fn display(&self) -> *mut libc::c_void {
        self.inner.conn.as_ref().unwrap().backend().display_ptr() as *mut libc::c_void
    }

    pub fn egl_window(&mut self, width: i32, height: i32) -> Result<*mut libc::c_void, OglError> {
        let object_id: &ObjectId = self.inner.surface.as_ref().unwrap().borrow();
        self.inner.egl_window = Some(
            WlEglSurface::new(object_id.to_owned(), width, height)
                .map_err(|e| Report::new(OglError::WaylandError).attach_printable(e))?,
        );

        Ok(self.inner.egl_window.as_ref().unwrap().ptr() as *mut libc::c_void)
    }

    pub fn dispatch(&mut self) -> Result<(), OglError> {
        //        let event_queue = self.event_queue.as_mut().unwrap();
        //
        //        let result = event_queue
        //            .blocking_dispatch(&mut self.inner)
        //            .map_err(|e| {
        //                Report::new(OglError::WaylandError)
        //                    .attach_printable(format!("Failed to dispatch: {e}"))
        //            })
        //            .map(|_| ());
        //
        //        result

        let event_queue = self.event_queue.as_mut().unwrap();
        loop {
            jtrace!("prepare_read");
            let Some(guard) = event_queue.prepare_read() else {
                event_queue
                    .dispatch_pending(&mut self.inner)
                    .map_err(|e| Report::new(OglError::WaylandError).attach_printable(e))?;
                continue;
            };

            jtrace!("Polling");
            let fd = guard.connection_fd();
            let mut fds = [rustix::event::PollFd::new(
                &fd,
                rustix::event::PollFlags::IN | rustix::event::PollFlags::ERR,
            )];

            loop {
                match rustix::event::poll(&mut fds, 1000) {
                    Ok(_) => break,
                    Err(rustix::io::Errno::INTR) => continue,
                    Err(e) => return Err(Report::new(OglError::WaylandError).attach_printable(e)),
                }
            }

            jtrace!("read");
            match guard.read() {
                Ok(n) => {
                    jtrace!("read {} events", n);
                    break;
                }
                Err(e) => return Err(Report::new(OglError::WaylandError).attach_printable(e)),
            }
        }

        Ok(())
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for WaylandStateInner {
    fn event(
        state: &mut Self,
        proxy: &wl_registry::WlRegistry,
        event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match &interface[..] {
                "wl_compositor" => {
                    jinfo!(
                        name = "WlRegistry",
                        event = "Dispatch",
                        interface = interface,
                        version = version,
                    );
                    let comp =
                        proxy.bind::<wl_compositor::WlCompositor, _, _>(name, version, qh, ());

                    let surface = comp.create_surface(qh, ());

                    state.comp = Some(comp);
                    state.surface = Some(surface);
                }
                "xdg_wm_base" => {
                    let wm_base = proxy.bind::<xdg_wm_base::XdgWmBase, _, _>(name, version, qh, ());
                    if let Some(wl_surface) = state.surface.as_ref() {
                        let s = wm_base.get_xdg_surface(wl_surface, qh, ());
                        let t = s.get_toplevel(qh, ());
                        t.set_title("xdg".into());
                        wl_surface.commit();

                        state.xdg_surface = Some((s, t));
                    }
                    state.xdg_base = Some(wm_base);
                }
                "wl_shm" => {
                    let shm = proxy.bind::<wl_shm::WlShm, _, _>(name, version, qh, ());
                    state.shm = Some(shm);
                }
                "wl_seat" => {
                    proxy.bind::<wl_seat::WlSeat, _, _>(name, version, qh, ());
                }

                _ => {
                    jdebug!(name = name, interface = interface, version = version);
                }
            }
        }
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for WaylandStateInner {
    fn event(
        _state: &mut Self,
        _proxy: &wl_compositor::WlCompositor,
        _event: <wl_compositor::WlCompositor as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        jdebug!(name = "WlRegistry", event = "Dispatch");
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for WaylandStateInner {
    fn event(
        _state: &mut Self,
        _proxy: &wl_surface::WlSurface,
        _event: <wl_surface::WlSurface as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        jinfo!(name = "WlSurface", event = "Dispatch");
    }
}

impl Dispatch<wl_shm::WlShm, ()> for WaylandStateInner {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm::WlShm,
        event: <wl_shm::WlShm as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let wl_shm::Event::Format { format } = event {
            match format {
                WEnum::Value(format) => {
                    jinfo!(
                        name = "WlShm",
                        event = "Dispatch",
                        format = format!("{:?}", format)
                    );
                }
                WEnum::Unknown(x) => {
                    jinfo!(name = "WlShm", event = "Dispatch", format = x,);
                }
            };
        } else {
            jinfo!(name = "WlShm", event = "Dispatch");
        }
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for WaylandStateInner {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm_pool::WlShmPool,
        _event: <wl_shm_pool::WlShmPool as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        jinfo!(name = "WlShmPool", event = "Dispatch");
    }
}

impl Dispatch<wl_buffer::WlBuffer, ()> for WaylandStateInner {
    fn event(
        _state: &mut Self,
        _proxy: &wl_buffer::WlBuffer,
        _event: <wl_buffer::WlBuffer as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        jinfo!(name = "WlBuffer", event = "Dispatch");
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for WaylandStateInner {
    fn event(
        _state: &mut Self,
        seat: &wl_seat::WlSeat,
        event: <wl_seat::WlSeat as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        jinfo!(name = "WlSeat", event = "Dispatch");
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(capabilities),
        } = event
        {
            if capabilities.contains(wl_seat::Capability::Keyboard) {
                seat.get_keyboard(qh, ());
            }
        }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for WaylandStateInner {
    fn event(
        state: &mut Self,
        _keyboard: &wl_keyboard::WlKeyboard,
        event: <wl_keyboard::WlKeyboard as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let wl_keyboard::Event::Key {
            key,
            state: WEnum::Value(key_state),
            ..
        } = event
        {
            if let Some(ref event_cb) = state.event_cb {
                match key_state {
                    KeyState::Pressed => {
                        if let Some(ref key_press) = event_cb.key_pressed {
                            key_press(key);
                        }
                    }
                    KeyState::Released => {
                        if let Some(ref key_released) = event_cb.key_released {
                            key_released(key);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for WaylandStateInner {
    fn event(
        _state: &mut Self,
        base: &xdg_wm_base::XdgWmBase,
        event: <xdg_wm_base::XdgWmBase as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        jinfo!(name = "XdgWmBase", event = "Dispatch");
        if let xdg_wm_base::Event::Ping { serial } = event {
            base.pong(serial);
        }
    }
}

impl Dispatch<xdg_surface::XdgSurface, ()> for WaylandStateInner {
    fn event(
        state: &mut Self,
        surface: &xdg_surface::XdgSurface,
        event: <xdg_surface::XdgSurface as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        jinfo!(name = "XdgSurface", event = "Dispatch");
        if let xdg_surface::Event::Configure { serial, .. } = event {
            surface.ack_configure(serial);
            state.configured = true;
        }
    }
}

impl Dispatch<xdg_toplevel::XdgToplevel, ()> for WaylandStateInner {
    fn event(
        _state: &mut Self,
        _proxy: &xdg_toplevel::XdgToplevel,
        _event: <xdg_toplevel::XdgToplevel as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        jinfo!(name = "XdgToplevel", event = "Dispatch");
    }
}
