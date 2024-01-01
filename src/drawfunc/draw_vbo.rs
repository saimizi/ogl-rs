use super::{DrawContext, DrawFunc, VertexOps};
use error_stack::{Report, Result};
use jlogger_tracing::jdebug;
use libogl::error::OglError;

pub fn draw_vbo(df: &mut DrawContext) -> Result<(), OglError> {
    if !df.initialized || df.draw_func != DrawFunc::DrawVbo {
        let v_src = include_str!("../../es300/sample.vert");
        let f_src = include_str!("../../es300/sample.frag");
        df.gl.build(Some(v_src), Some(f_src))?;

        df.initialized = true;
        df.draw_func = DrawFunc::DrawVbo;
    }

    let gl = df.gl.gl();
    let program = df.gl.program().ok_or(Report::new(OglError::InvalidData))?;

    unsafe {
        gl.Viewport(0, 0, df.width, df.height);
        gl.ClearColor(0.07_f32, 0.13_f32, 0.17_f32, 1.0_f32);
        gl.Clear(gl33::GL_COLOR_BUFFER_BIT);
        gl.UseProgram(program);

        // Use uniform to set color for all vertex in frag shader.

        let name = std::ffi::CString::new("u_Color").unwrap();
        let location = gl.GetUniformLocation(program, name.as_ptr().cast());

        gl.Uniform4f(location, 0.8f32, 0.3f32, 0.02f32, 1.0f32);

        if df.vbo[0] == 0 {
            #[rustfmt::skip]
                let vertices = [
                    0.0f32,    0.5f32, 0.0f32,
                   -0.5f32,   -0.5f32, 0.0f32,
                    0.5f32,   -0.5f32, 0.0f32,
                ];

            gl.GenBuffers(1, &mut df.vbo as *mut u32);
            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[0]);

            let vertices_u8 = vertices.to_u8_slice();
            gl.BufferData(
                gl33::GL_ARRAY_BUFFER,
                vertices_u8.len() as isize,
                vertices_u8.as_ptr().cast(),
                gl33::GL_STATIC_DRAW,
            );

            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, 0);
        }

        jdebug!("drawing with VBO");
        gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[0]);
        gl.VertexAttribPointer(0, 3, gl33::GL_FLOAT, 0, 0, core::ptr::null_mut());
        gl.EnableVertexAttribArray(0);
        gl.DrawArrays(gl33::GL_TRIANGLES, 0, 3);
        gl.BindBuffer(gl33::GL_ARRAY_BUFFER, 0);
        gl.DisableVertexAttribArray(0);
        gl.Flush();
    }
    Ok(())
}
