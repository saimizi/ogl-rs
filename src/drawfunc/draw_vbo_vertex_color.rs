use super::{DrawContext, DrawFunc};
use error_stack::Result;
use jlogger_tracing::jdebug;
use libogl::error::OglError;
use libogl::VertexOps;

pub fn draw_vbo_vertex_color(df: &mut DrawContext) -> Result<(), OglError> {
    if !df.initialized || df.draw_func != DrawFunc::DrawVboVertexColor {
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
        df.draw_func = DrawFunc::DrawVboVertexColor;

        df.gl.gl().UseProgram(df.gl.program().unwrap());
    }

    unsafe {
        let gl = df.gl.gl();

        gl.Viewport(0, 0, df.width, df.height);
        gl.ClearColor(0.07_f32, 0.13_f32, 0.17_f32, 1.0_f32);
        gl.Clear(gl33::GL_COLOR_BUFFER_BIT);

        // Use a single vertex buffer object to store both vertex and color data.
        if df.vbo[0] == 0 {
            #[rustfmt::skip]
            let vertices = [
                0.0f32,    0.5f32, 0.0f32,          1.0f32, 0.0f32, 0.0f32,
               -0.5f32,   -0.5f32, 0.0f32,          0.0f32, 1.0f32, 0.0f32,
                0.5f32,   -0.5f32, 0.0f32,          0.0f32, 0.0f32, 1.0f32,
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

        jdebug!("drawing");
        gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[0]);

        // Set vertex attribute (vPosition)
        gl.EnableVertexAttribArray(0);

        gl.VertexAttribPointer(
            0,
            3,
            gl33::GL_FLOAT,
            0,
            (core::mem::size_of::<f32>() * 6) as i32,
            0 as *const std::ffi::c_void,
        );

        // Set color vertex attribute (vColor)
        gl.EnableVertexAttribArray(1);

        // Setting "pointer" is tricky.
        //   According to the specification, "pointer" is a raw pointer pointing to the first
        // generic vertex attribute in the array, but should be an offset which is cased to
        // pointer when using VBO.In Rust, we can do it with unsafe.
        //   It seems that something like following does also work
        //     (core::ptr::null() as *const u8).add(core::mem::size_of::<f32>() * 3) as
        //         *const std::ffi::c_void
        gl.VertexAttribPointer(
            1,
            3,
            gl33::GL_FLOAT,
            0,
            (core::mem::size_of::<f32>() * 6) as i32,
            (core::mem::size_of::<f32>() * 3) as *const std::ffi::c_void,
        );

        gl.DrawArrays(gl33::GL_TRIANGLES, 0, 3);

        gl.DisableVertexAttribArray(0);
        gl.DisableVertexAttribArray(1);
        gl.BindBuffer(gl33::GL_ARRAY_BUFFER, 0);

        gl.Flush();
    }
    Ok(())
}
