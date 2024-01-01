pub mod drawfunc;
pub mod egl;
pub mod gl;
pub mod sdl;
pub mod wayland;

#[allow(unused)]
use {
    clap::{Args, Parser},
    core::borrow::Borrow,
    drawfunc::{DrawContext, DrawContextOps, DrawFunc, RunState},
    egl::EglState,
    error_stack::{Report, Result, ResultExt},
    gl::GlState,
    jlogger_tracing::{
        jdebug, jerror, jinfo, jtrace, jwarn, JloggerBuilder, LevelFilter, LogTimeFormat,
    },
    libm::sqrt,
    libogl::error::OglError,
    sdl::Sdl2State,
    std::f64::consts::PI,
    std::sync::atomic::{AtomicBool, Ordering},
    std::{fs::File, os::fd::AsFd},
    std::{thread::sleep, time::Duration},
    wayland::{WaylandEventCb, WaylandState},
    wayland_client::{
        protocol::wl_keyboard::{self, KeyState},
        protocol::{
            wl_buffer, wl_compositor, wl_registry, wl_seat, wl_shm, wl_shm_pool, wl_surface,
        },
        Connection, Dispatch, EventQueue, Proxy, QueueHandle, WEnum,
    },
    wayland_egl::WlEglSurface,
    wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base},
};

#[derive(Parser)]
#[command(author, version, about, long_about= None)]
struct Cli {
    #[arg(short, long, default_value_t = String::from("800x800"))]
    window: String,

    #[command(flatten)]
    backend: Backend,

    #[arg(short, long, default_value_t = 0usize)]
    func: usize,

    #[arg(short, long, action=clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct Backend {
    #[arg(short = 'W', long)]
    wayland: bool,

    #[arg(short = 'S', long)]
    sdl: bool,
}

struct WaylandOps {
    pub ws: WaylandState,
    pub egl: EglState,
}

impl DrawContextOps for WaylandOps {
    fn do_dispatch(&mut self) -> Result<(), OglError> {
        self.ws.dispatch()
    }
    fn do_swap(&self) -> Result<(), OglError> {
        self.egl.swap_buffers()
    }
}

impl DrawContextOps for Sdl2State {
    fn do_dispatch(&mut self) -> Result<(), OglError> {
        self.dispatch()
    }

    fn do_swap(&self) -> Result<(), OglError> {
        self.swap_window()
    }
}

fn main() -> Result<(), OglError> {
    let cli = Cli::parse();

    let level = match cli.verbose {
        1 => LevelFilter::DEBUG,
        2 => LevelFilter::TRACE,
        _ => LevelFilter::INFO,
    };

    JloggerBuilder::new()
        .max_level(level)
        .log_time(LogTimeFormat::TimeStamp)
        .build();

    let w: Vec<i32> = cli
        .window
        .as_str()
        .split('x')
        .map(|a| a.parse::<i32>().unwrap())
        .collect();
    assert_eq!(w.len(), 2);

    let width = w[0];
    let height = w[1];

    if cli.backend.wayland {
        let ws_cb = WaylandEventCb {
            key_pressed: Some(Box::new(|key: u32| {
                if key == 1 {
                    RunState::global_stop()
                }
            })),
            key_released: None,
        };

        let mut ws = WaylandState::new(Some(ws_cb))?;
        let egl = EglState::new(ws.display(), ws.egl_window(width, height)?)?;
        let gl = GlState::new(&egl, None, None)?;
        egl.swap_interval(1)?;

        let mut dt = DrawContext::new(gl, width, height);
        let mut w = WaylandOps { ws, egl };

        dt.run(&mut w, cli.func.into())?;
    } else if cli.backend.sdl {
        let mut sdl = Sdl2State::new(width, height)?;
        let gl = GlState::new(&sdl, None, None)?;
        let mut dt = DrawContext::new(gl, width, height);

        dt.run(&mut sdl, cli.func.into())?;
    }

    Ok(())
}
