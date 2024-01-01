#[allow(unused)]
use {
    super::gl::GlContextOps,
    clap::{Args, Parser},
    core::borrow::Borrow,
    error_stack::{Report, Result, ResultExt},
    jlogger_tracing::{
        jdebug, jerror, jinfo, jtrace, jwarn, JloggerBuilder, LevelFilter, LogTimeFormat,
    },
    libm::sqrt,
    libogl::error::OglError,
    sdl2::{
        event::Event,
        video::{GLContext, Window},
        Sdl, VideoSubsystem,
    },
    std::f64::consts::PI,
    std::sync::atomic::{AtomicBool, Ordering},
    std::{fs::File, os::fd::AsFd},
    std::{thread::sleep, time::Duration},
};

pub struct Sdl2State {
    context: Sdl,
    video: VideoSubsystem,
    window: Window,
    _gl_context: GLContext,
}

impl GlContextOps for Sdl2State {
    fn get_proc_address(&self, s: &str) -> *mut std::ffi::c_void {
        self.video.gl_get_proc_address(s) as *mut std::ffi::c_void
    }
}

impl Sdl2State {
    pub fn new(width: i32, height: i32) -> Result<Self, OglError> {
        let context =
            sdl2::init().map_err(|e| Report::new(OglError::SDLError).attach_printable(e))?;

        let video = context
            .video()
            .map_err(|e| Report::new(OglError::SDLError).attach_printable(e))?;

        let gl_attr = video.gl_attr();
        gl_attr.set_context_version(3, 0);
        #[cfg(target_arch = "x86_64")]
        {
            jinfo!("Using Core profile.");
            gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        }

        #[cfg(target_arch = "aarch64")]
        {
            jinfo!("Using GLES profile.");
            gl_attr.set_context_profile(sdl2::video::GLProfile::GLES);
        }

        let window = video
            .window("OGLWIN", width as u32, height as u32)
            .opengl()
            .position_centered()
            .build()
            .map_err(|e| Report::new(OglError::SDLError).attach_printable(e))?;

        let _gl_context = window
            .gl_create_context()
            .map_err(|e| Report::new(OglError::SDLError).attach_printable(e))?;

        Ok(Self {
            context,
            video,
            window,
            _gl_context,
        })
    }

    pub fn dispatch(&self) -> Result<(), OglError> {
        let mut event_pump = self
            .context
            .event_pump()
            .map_err(|e| Report::new(OglError::SDLError).attach_printable(e))?;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { timestamp: _ } => {
                    super::RunState::global_stop();
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if key == sdl2::keyboard::Keycode::Escape {
                        super::RunState::global_stop();
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn swap_window(&self) -> Result<(), OglError> {
        self.window.gl_swap_window();
        Ok(())
    }
}
