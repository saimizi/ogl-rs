use super::{DrawContext, DrawFunc};
use error_stack::{Report, Result};
use jlogger_tracing::jdebug;
use libogl::error::OglError;
use libogl::VertexOps;

pub fn draw_lines(df: &mut DrawContext) -> Result<(), OglError> {
    if !df.initialized || df.draw_func != DrawFunc::DrawLines {
        let v_src = r#"
                #version 300 es
                layout(location = 0) in vec4 vPosition;
                layout(location = 1) in vec4 vColor;

                out vec4 vColorVec;

                void main()
                {
                    gl_Position = vPosition;
                    vColorVec = vColor;

                }
        "#;

        let f_src = r#"
                #version 300 es
                precision mediump float;
                out vec4 fragColor;

                in vec4 vColorVec;
                void main()
                {
                    fragColor = vColorVec ;
                }
        "#;

        df.gl.build(Some(v_src), Some(f_src))?;

        df.initialized = true;
        df.draw_func = DrawFunc::DrawLines;
    }

    let gl = df.gl.gl();
    let program = df.gl.program().ok_or(Report::new(OglError::InvalidData))?;

    unsafe {
        gl.Viewport(0, 0, df.width, df.height);
        gl.ClearColor(0.07_f32, 0.13_f32, 0.17_f32, 1.0_f32);
        gl.Clear(gl33::GL_COLOR_BUFFER_BIT);
        gl.UseProgram(program);

        if df.vbo[0] == 0 {
            #[rustfmt::skip]
                let vertices = [
                    0.0f32,    f32::sqrt(0.5f32), 0.0f32,             1.0f32, 0.0f32, 0.0f32,
                   -0.5f32,   -0.5f32,                  0.0f32,             0.0f32, 1.0f32, 0.0f32,
                    0.5f32,   -0.5f32,                  0.0f32,             0.0f32, 0.0f32, 1.0f32,
                    0.0f32,   -0.0f32,                  0.0f32,             1.0f32, 1.0f32, 1.0f32,
                ];

            gl.GenBuffers(3, &mut df.vbo as *mut u32);
            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[0]);

            let vertices_u8 = vertices.to_u8_slice();
            gl.BufferData(
                gl33::GL_ARRAY_BUFFER,
                vertices_u8.len() as isize,
                vertices_u8.as_ptr().cast(),
                gl33::GL_STATIC_DRAW,
            );

            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, 0);

            #[rustfmt::skip]
                let indices_u8 = [
                    0u8, 3u8, 1u8, 3u8, 2u8, 3u8
                ];

            gl.BindBuffer(gl33::GL_ELEMENT_ARRAY_BUFFER, df.vbo[1]);
            gl.BufferData(
                gl33::GL_ELEMENT_ARRAY_BUFFER,
                indices_u8.len() as isize,
                indices_u8.as_ptr().cast(),
                gl33::GL_STATIC_DRAW,
            );
            gl.BindBuffer(gl33::GL_ELEMENT_ARRAY_BUFFER, 0);

            #[rustfmt::skip]
                let indices_u8 = [
                    0u8, 1u8, 2u8,
                ];

            gl.BindBuffer(gl33::GL_ELEMENT_ARRAY_BUFFER, df.vbo[2]);
            gl.BufferData(
                gl33::GL_ELEMENT_ARRAY_BUFFER,
                indices_u8.len() as isize,
                indices_u8.as_ptr().cast(),
                gl33::GL_STATIC_DRAW,
            );
            gl.BindBuffer(gl33::GL_ELEMENT_ARRAY_BUFFER, 0);
        }

        jdebug!("drawing with VBO");
        let stride = core::mem::size_of::<f32>() * 6;
        let mut offset = 0;
        gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[0]);
        gl.EnableVertexAttribArray(0);
        gl.VertexAttribPointer(
            0,
            3,
            gl33::GL_FLOAT,
            0,
            stride as i32,
            offset as *const std::ffi::c_void,
        );

        offset = core::mem::size_of::<f32>() * 3;
        gl.EnableVertexAttribArray(1);
        gl.VertexAttribPointer(
            1,
            3,
            gl33::GL_FLOAT,
            0,
            stride as i32,
            offset as *const std::ffi::c_void,
        );

        gl.LineWidth(2.0f32);
        gl.BindBuffer(gl33::GL_ELEMENT_ARRAY_BUFFER, df.vbo[1]);
        gl.DrawElements(
            gl33::GL_LINE_LOOP,
            6,
            gl33::GL_UNSIGNED_BYTE,
            0 as *const std::ffi::c_void,
        );

        gl.LineWidth(5.0f32);
        gl.BindBuffer(gl33::GL_ELEMENT_ARRAY_BUFFER, df.vbo[2]);
        gl.DrawElements(
            gl33::GL_LINE_STRIP,
            6,
            gl33::GL_UNSIGNED_BYTE,
            0 as *const std::ffi::c_void,
        );

        gl.BindBuffer(gl33::GL_ARRAY_BUFFER, 0);
        gl.BindBuffer(gl33::GL_ELEMENT_ARRAY_BUFFER, 0);
        gl.DisableVertexAttribArray(0);
        gl.Flush();
    }
    Ok(())
}
