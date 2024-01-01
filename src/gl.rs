#[allow(unused)]
use {
    super::egl::EglState,
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

pub trait GlContextOps {
    fn get_proc_address(&self, s: &str) -> *mut std::ffi::c_void;
}

pub struct GlState {
    gl: gl33::GlFns,
    program: Option<u32>,
    v_src: String,
    f_src: String,
}

impl GlState {
    pub fn new(
        egl: &dyn GlContextOps,
        v_src: Option<&str>,
        f_src: Option<&str>,
    ) -> Result<Self, OglError> {
        let gl = unsafe {
            gl33::GlFns::load_from(&|p| {
                let s = std::ffi::CStr::from_ptr(p.cast()).to_str().unwrap();
                egl.get_proc_address(s)
            })
            .unwrap()
        };

        let mut state = Self {
            gl,
            v_src: String::new(),
            f_src: String::new(),
            program: None,
        };

        if v_src.is_some() && f_src.is_some() {
            state.build(v_src, f_src)?;
        }

        Ok(state)
    }

    pub fn build(&mut self, v_src: Option<&str>, f_src: Option<&str>) -> Result<(), OglError> {
        if let Some(s) = v_src {
            self.v_src = s.to_owned();
        }

        if let Some(s) = f_src {
            self.f_src = s.to_owned();
        }

        unsafe {
            let gl = &mut self.gl;

            let v_shader = gl.CreateShader(gl33::GL_VERTEX_SHADER);
            gl.ShaderSource(
                v_shader,
                1,
                &self.v_src.as_bytes().as_ptr().cast(),
                &self.v_src.len().try_into().unwrap(),
            );

            gl.CompileShader(v_shader);

            let mut success = 0;
            gl.GetShaderiv(v_shader, gl33::GL_COMPILE_STATUS, &mut success);

            if success == 0 {
                let mut v: Vec<u8> = Vec::with_capacity(1024);
                let mut log_len = 0_i32;

                gl.GetShaderInfoLog(v_shader, 1024, &mut log_len, v.as_mut_ptr().cast());

                v.set_len(log_len.try_into().unwrap());
                let error_msg = String::from_utf8_lossy(&v).to_string();
                jerror!("Error: {}", error_msg);

                return Err(Report::new(OglError::GlError).attach_printable(error_msg));
            }

            let f_shader = gl.CreateShader(gl33::GL_FRAGMENT_SHADER);

            gl.ShaderSource(
                f_shader,
                1,
                &self.f_src.as_bytes().as_ptr().cast(),
                &self.f_src.len().try_into().unwrap(),
            );

            gl.CompileShader(f_shader);

            gl.GetShaderiv(f_shader, gl33::GL_COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut v: Vec<u8> = Vec::with_capacity(1024);
                let mut log_len = 0_i32;

                gl.GetShaderInfoLog(f_shader, 1024, &mut log_len, v.as_mut_ptr().cast());

                v.set_len(log_len.try_into().unwrap());

                return Err(Report::new(OglError::GlError)
                    .attach_printable(String::from_utf8_lossy(&v).to_string()));
            }

            let program = gl.CreateProgram();
            gl.AttachShader(program, v_shader);
            gl.AttachShader(program, f_shader);
            gl.LinkProgram(program);

            gl.GetProgramiv(program, gl33::GL_LINK_STATUS, &mut success);
            if success == 0 {
                let mut v: Vec<u8> = Vec::with_capacity(1024);
                let mut log_len = 0_i32;

                gl.GetProgramInfoLog(program, 1024, &mut log_len, v.as_mut_ptr().cast());

                v.set_len(log_len.try_into().unwrap());

                return Err(Report::new(OglError::GlError)
                    .attach_printable(String::from_utf8_lossy(&v).to_string()));
            }

            self.program = Some(program);
        }

        Ok(())
    }

    pub fn gl(&self) -> &gl33::GlFns {
        &self.gl
    }

    pub fn program(&self) -> Option<u32> {
        self.program.as_ref().cloned()
    }
}
