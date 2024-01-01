pub mod draw_circle;
pub mod draw_complex;
pub mod draw_instance;
pub mod draw_instance2;
pub mod draw_lines;
pub mod draw_model_view_projection;
pub mod draw_primitive_restart;
pub mod draw_provoking_vertex;
pub mod draw_triangle_strip;
pub mod draw_vao_elements;
pub mod draw_vao_vertex_color;
pub mod draw_vao_vertex_color2;
pub mod draw_vbo;
pub mod draw_vbo2;
pub mod draw_vbo_vertex_color;
pub mod draw_vbo_vertex_color2;
pub mod draw_without_vbo;

use super::gl::GlState;
use error_stack::Result;
use jlogger_tracing::jinfo;
use libogl::error::OglError;
use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use draw_circle::draw_circle;
use draw_complex::draw_complex;
use draw_instance::draw_instance;
use draw_instance2::draw_instance2;
use draw_lines::draw_lines;
use draw_model_view_projection::draw_model_view_projection;
use draw_primitive_restart::draw_primitive_restart;
use draw_provoking_vertex::draw_provoking_vertex;
use draw_triangle_strip::draw_triangle_strip;
use draw_vao_elements::draw_vao_elements;
use draw_vao_vertex_color::draw_vao_vertex_color;
use draw_vao_vertex_color2::draw_vao_vertex_color2;
use draw_vbo::draw_vbo;
use draw_vbo2::draw_vbo2;
use draw_vbo_vertex_color::draw_vbo_vertex_color;
use draw_vbo_vertex_color2::draw_vbo_vertex_color2;
use draw_without_vbo::draw_without_vbo;

static START: OnceCell<Instant> = once_cell::sync::OnceCell::new();
pub fn elapsed_seconds() -> u64 {
    START.get().unwrap().elapsed().as_secs()
}

pub fn elapsed_milliseconds() -> u128 {
    START.get().unwrap().elapsed().as_millis()
}

static mut RUNNING: AtomicBool = AtomicBool::new(false);

pub struct RunState {}

impl RunState {
    pub fn global_run() {
        unsafe {
            START.set(Instant::now()).unwrap();
            RUNNING.store(true, Ordering::Relaxed);
        }
    }
    pub fn global_stop() {
        unsafe {
            RUNNING.store(false, Ordering::Relaxed);
        }
    }

    pub fn is_running() -> bool {
        unsafe { RUNNING.load(Ordering::Relaxed) }
    }
}

trait VertexOps {
    fn to_u8_slice(&self) -> &[u8];
}

impl VertexOps for [f32] {
    fn to_u8_slice(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self.as_ptr() as *const u8,
                self.len() * core::mem::size_of::<f32>(),
            )
        }
    }
}

impl VertexOps for [u16] {
    fn to_u8_slice(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self.as_ptr() as *const u8,
                self.len() * core::mem::size_of::<u16>(),
            )
        }
    }
}

impl VertexOps for Vec<f32> {
    fn to_u8_slice(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self.as_ptr() as *const u8,
                self.len() * core::mem::size_of::<f32>(),
            )
        }
    }
}

pub trait DrawContextOps {
    fn do_dispatch(&mut self) -> Result<(), OglError>;
    fn do_swap(&self) -> Result<(), OglError>;
}

#[allow(unused)]
#[derive(Debug, PartialEq, PartialOrd)]
pub enum DrawFunc {
    DrawVbo,
    DrawVbo2,
    DrawVboVertexColor,
    DrawVboVertexColor2,
    DrawVaoVertexColor,
    DrawVaoVertexColor2,
    DrawVaoVertexColorElement2,
    DrawCircle,
    DrawComplex,
    DrawWithoutVbo,
    DrawLines,
    DrawPrimitiveRestart,
    DrawProvokingVertex,
    DrawInstance,
    DrawInstance2,
    DrawTriangleStrip,
    DrawModelViewProjection,
}

impl std::fmt::Display for DrawFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let index: usize = self.into();

        let msg = match self {
            DrawFunc::DrawVbo => format!("{:3}_DrawVbo", index),
            DrawFunc::DrawVbo2 => format!("{:3}_DrawVbo2", index),
            DrawFunc::DrawVboVertexColor => format!("{}_DrawVboVertexColor", index),
            DrawFunc::DrawVboVertexColor2 => {
                format!("{}_DrawVboVertexColor2", index)
            }
            DrawFunc::DrawVaoVertexColor => format!("{}_DrawVaoVertexColor", index),
            DrawFunc::DrawVaoVertexColor2 => {
                format!("{}_DrawVaoVertexColor2", index)
            }
            DrawFunc::DrawVaoVertexColorElement2 => {
                format!("{}, DrawVaoVertexColorElement2", index)
            }
            DrawFunc::DrawCircle => format!("{}_DrawCircle", index),
            DrawFunc::DrawComplex => format!("{}_DrawComplex", index),
            DrawFunc::DrawWithoutVbo => format!("{}_DrawWithoutVbo", index),
            DrawFunc::DrawLines => format!("{}_DrawLines", index),
            DrawFunc::DrawPrimitiveRestart => {
                format!("{}_DrawPrimitiveRestart", index)
            }
            DrawFunc::DrawProvokingVertex => {
                format!("{}_DrawProvokingVertex", index)
            }
            DrawFunc::DrawInstance => format!("{}_DrawInstance", index),
            DrawFunc::DrawInstance2 => format!("{}_DrawInstance2", index),
            DrawFunc::DrawTriangleStrip => format!("{}_DrawTriangleStrip", index),
            DrawFunc::DrawModelViewProjection => format!("{}_DrawModelViewProjection", index),
        };

        write!(f, "{}", msg)
    }
}

impl From<usize> for DrawFunc {
    fn from(value: usize) -> Self {
        match value {
            1 => DrawFunc::DrawVbo,
            2 => DrawFunc::DrawVbo2,
            3 => DrawFunc::DrawVboVertexColor,
            4 => DrawFunc::DrawVboVertexColor2,
            5 => DrawFunc::DrawVaoVertexColor,
            6 => DrawFunc::DrawVaoVertexColor2,
            7 => DrawFunc::DrawVaoVertexColorElement2,
            8 => DrawFunc::DrawCircle,
            9 => DrawFunc::DrawComplex,
            10 => DrawFunc::DrawWithoutVbo,
            11 => DrawFunc::DrawLines,
            12 => DrawFunc::DrawPrimitiveRestart,
            13 => DrawFunc::DrawProvokingVertex,
            14 => DrawFunc::DrawInstance,
            15 => DrawFunc::DrawInstance2,
            16 => DrawFunc::DrawTriangleStrip,
            17 => DrawFunc::DrawModelViewProjection,
            _ => DrawFunc::DrawModelViewProjection,
        }
    }
}

impl From<&DrawFunc> for usize {
    fn from(value: &DrawFunc) -> Self {
        (1..=100)
            .into_iter()
            .find(|a| &DrawFunc::from(*a) == value)
            .unwrap()
    }
}

#[allow(unused)]
pub struct DrawContext {
    w: i32,
    h: i32,
    turn_small: bool,
    width: i32,
    height: i32,
    gl: GlState,
    vao: Option<u32>,
    vbo: [u32; 16],
    locations: [i32; 256],
    vertex_number: u32,
    initialized: bool,
    draw_func: DrawFunc,
}

impl DrawContext {
    pub fn new(gl: GlState, width: i32, height: i32) -> Self {
        Self {
            w: 0,
            h: 0,
            turn_small: false,
            width,
            height,
            gl,
            vao: None,
            vbo: [0_u32; 16],
            locations: [0; 256],
            vertex_number: 0,
            initialized: false,
            draw_func: DrawFunc::DrawVbo,
        }
    }

    pub fn run(
        &mut self,
        ops: &mut dyn DrawContextOps,
        draw_func: DrawFunc,
    ) -> Result<(), OglError> {
        RunState::global_run();

        jinfo!(func = draw_func.to_string());
        while RunState::is_running() {
            ops.do_dispatch()?;

            match draw_func {
                DrawFunc::DrawVbo => draw_vbo(self)?,
                DrawFunc::DrawVbo2 => draw_vbo2(self)?,
                DrawFunc::DrawLines => draw_lines(self)?,
                DrawFunc::DrawCircle => draw_circle(self)?,
                DrawFunc::DrawComplex => draw_complex(self)?,
                DrawFunc::DrawWithoutVbo => draw_without_vbo(self)?,
                DrawFunc::DrawVboVertexColor => draw_vbo_vertex_color(self)?,
                DrawFunc::DrawVboVertexColor2 => draw_vbo_vertex_color2(self)?,
                DrawFunc::DrawVaoVertexColor => draw_vao_vertex_color(self)?,
                DrawFunc::DrawVaoVertexColor2 => draw_vao_vertex_color2(self)?,
                DrawFunc::DrawVaoVertexColorElement2 => draw_vao_elements(self)?,
                DrawFunc::DrawPrimitiveRestart => draw_primitive_restart(self)?,
                DrawFunc::DrawProvokingVertex => draw_provoking_vertex(self)?,
                DrawFunc::DrawInstance => draw_instance(self)?,
                DrawFunc::DrawInstance2 => draw_instance2(self)?,
                DrawFunc::DrawTriangleStrip => draw_triangle_strip(self)?,
                DrawFunc::DrawModelViewProjection => draw_model_view_projection(self)?,
            }

            ops.do_swap()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::DrawFunc;

    #[test]
    fn drawfunc_to_usize() {
        let index: usize = (&DrawFunc::DrawVbo).into();
        assert_eq!(index, 1);

        let index: usize = (&DrawFunc::DrawModelViewProjection).into();
        assert_eq!(index, 17);
    }
}
