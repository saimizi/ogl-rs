use super::{DrawContext, DrawFunc};
use error_stack::{Report, Result};
use jlogger_tracing::jdebug;
use libogl::error::OglError;
use libogl::VertexOps;

pub fn draw_without_vbo(df: &mut DrawContext) -> Result<(), OglError> {
    if !df.initialized || df.draw_func != DrawFunc::DrawWithoutVbo {
        let v_src = include_str!("../../es300/sample.vert");
        let f_src = include_str!("../../es300/sample.frag");
        df.gl.build(Some(v_src), Some(f_src))?;

        df.initialized = true;
        df.draw_func = DrawFunc::DrawWithoutVbo;
    }

    let gl = df.gl.gl();
    let program = df.gl.program().ok_or(Report::new(OglError::InvalidData))?;

    unsafe {
        gl.Viewport(0, 0, df.width, df.height);
        gl.ClearColor(0.07_f32, 0.13_f32, 0.17_f32, 1.0_f32);
        gl.Clear(gl33::GL_COLOR_BUFFER_BIT);
        gl.UseProgram(program);

        let name = std::ffi::CString::new("u_Color").unwrap();
        let location = gl.GetUniformLocation(program, name.as_ptr().cast());
        gl.Uniform4f(location, 0.8f32, 0.3f32, 0.02f32, 1.0f32);

        #[rustfmt::skip]
        let vertices: [f32; 9] = [
                0.0f32,    0.5f32, 0.0f32,
               -0.5f32,   -0.5f32, 0.0f32,
                0.5f32,   -0.5f32, 0.0f32,
        ];

        jdebug!("drawing without VBO.");

        gl.VertexAttribPointer(
            0,
            3,
            gl33::GL_FLOAT,
            0,
            0,
            vertices.to_u8_slice().as_ptr().cast(),
        );

        gl.EnableVertexAttribArray(0);
        gl.DrawArrays(gl33::GL_TRIANGLES, 0, 3);
        gl.DisableVertexAttribArray(0);
    }
    Ok(())
}
