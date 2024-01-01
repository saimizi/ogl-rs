#[allow(unused)]
use {
    super::gl::GlContextOps,
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
pub struct EglState {
    egl: Option<khronos_egl::DynamicInstance<khronos_egl::EGL1_4>>,
    egl_display: Option<khronos_egl::Display>,
    egl_surface: Option<khronos_egl::Surface>,
    egl_context: Option<khronos_egl::Context>,
}

impl EglState {
    pub fn new(
        native_display: *mut libc::c_void,
        native_window: *mut libc::c_void,
    ) -> Result<Self, OglError> {
        assert_ne!(native_display, core::ptr::null_mut());
        assert_ne!(native_window, core::ptr::null_mut());

        let lib = unsafe { libloading::Library::new("libEGL.so.1").unwrap() };
        let egl = unsafe {
            khronos_egl::DynamicInstance::<khronos_egl::EGL1_4>::load_required_from(lib).unwrap()
        };

        let egl_display = unsafe {
            egl.get_display(native_display)
                .ok_or(Report::new(OglError::EglError).attach("Failed to get EGL display"))?
        };

        egl.initialize(egl_display)
            .map_err(|e| Report::new(OglError::EglError).attach_printable(format!("{e}")))?;

        egl.bind_api(khronos_egl::OPENGL_ES_API)
            .map_err(|e| Report::new(OglError::EglError).attach_printable(format!("{e}")))?;

        let config_attributes = [
            khronos_egl::SURFACE_TYPE,
            khronos_egl::WINDOW_BIT,
            khronos_egl::RED_SIZE,
            8,
            khronos_egl::GREEN_SIZE,
            8,
            khronos_egl::BLUE_SIZE,
            8,
            khronos_egl::ALPHA_SIZE,
            8,
            khronos_egl::RENDERABLE_TYPE,
            khronos_egl::OPENGL_ES3_BIT,
            khronos_egl::NONE,
        ];

        let context_attribute = [khronos_egl::CONTEXT_CLIENT_VERSION, 2, khronos_egl::NONE];

        let config = egl
            .choose_first_config(egl_display, &config_attributes)
            .map_err(|e| Report::new(OglError::EglError).attach_printable(format!("{e}")))?
            .ok_or(Report::new(OglError::EglError).attach_printable("No usable config found"))?;

        let egl_context = egl
            .create_context(egl_display, config, None, &context_attribute)
            .map_err(|e| Report::new(OglError::EglError).attach_printable(format!("{e}")))?;

        let egl_surface = unsafe {
            egl.create_window_surface(egl_display, config, native_window, None)
                .map_err(|e| Report::new(OglError::EglError).attach_printable(format!("{e}")))?
        };

        egl.make_current(
            egl_display,
            Some(egl_surface),
            Some(egl_surface),
            Some(egl_context),
        )
        .map_err(|e| Report::new(OglError::EglError).attach_printable(format!("{e}")))?;

        jinfo!("EGL initialized");
        Ok(Self {
            egl: Some(egl),
            egl_display: Some(egl_display),
            egl_context: Some(egl_context),
            egl_surface: Some(egl_surface),
        })
    }

    pub fn swap_buffers(&self) -> Result<(), OglError> {
        let egl = self.egl.as_ref().unwrap();
        let display = self.egl_display.as_ref().unwrap();
        let surface = self.egl_surface.as_ref().unwrap();

        egl.swap_buffers(display.to_owned(), surface.to_owned())
            .map_err(|e| Report::new(OglError::EglError).attach_printable(format!("{e}")))
    }

    pub fn swap_interval(&self, interval: i32) -> Result<(), OglError> {
        let egl = self.egl.as_ref().unwrap();
        let display = self.egl_display.as_ref().unwrap();

        egl.swap_interval(display.to_owned(), interval)
            .map_err(|e| Report::new(OglError::EglError).attach_printable(format!("{e}")))
    }

    pub fn make_current(&self) -> Result<(), OglError> {
        let egl = self.egl.as_ref().unwrap();
        let display = self.egl_display.as_ref().unwrap();
        let surface = self.egl_surface.as_ref().unwrap();
        let context = self.egl_context.as_ref().unwrap();

        egl.make_current(
            display.to_owned(),
            Some(surface.to_owned()),
            Some(surface.to_owned()),
            Some(context.to_owned()),
        )
        .map_err(|e| Report::new(OglError::EglError).attach_printable(format!("{e}")))
    }
}

impl GlContextOps for EglState {
    fn get_proc_address(&self, s: &str) -> *mut std::ffi::c_void {
        self.egl
            .as_ref()
            .unwrap()
            .get_proc_address(s)
            .expect(&format!("Failed to load {s}")) as *mut std::ffi::c_void
    }
}
