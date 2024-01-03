use super::{DrawContext, DrawFunc};
use error_stack::Result;
use jlogger_tracing::jinfo;
use libogl::error::OglError;
use libogl::VertexOps;

pub fn draw_primitive_restart(df: &mut DrawContext) -> Result<(), OglError> {
    static mut INDICES_NUM: i32 = 0;
    if !df.initialized || df.draw_func != DrawFunc::DrawVaoVertexColorElement2 {
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
        df.draw_func = DrawFunc::DrawVaoVertexColorElement2;

        unsafe {
            let gl = df.gl.gl();
            gl.UseProgram(df.gl.program().unwrap());

            #[rustfmt::skip]
                let vertices = [
                   -0.5f32,    0.5f32, 0.0f32,      // 0
                    0.5f32,    0.5f32, 0.0f32,      // 1
                    0.0f32,    0.1f32, 0.0f32,      // 2
                    0.1f32,    0.0f32, 0.0f32,      // 3
                    0.0f32,    0.0f32, 0.0f32,      // 4
                   -0.1f32,    0.0f32, 0.0f32,      // 5
                    0.0f32,   -0.1f32, 0.0f32,      // 6
                   -0.5f32,   -0.5f32, 0.0f32,      // 7
                    0.5f32,   -0.5f32, 0.0f32,      // 8
                ];

            // Create VBO for vertex and color
            gl.GenBuffers(3, &mut df.vbo as *mut u32);
            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[0]);

            let vertices_u8 = vertices.to_u8_slice();
            gl.BufferData(
                gl33::GL_ARRAY_BUFFER,
                vertices_u8.len() as isize,
                vertices_u8.as_ptr().cast(),
                gl33::GL_STATIC_DRAW,
            );

            #[rustfmt::skip]
                let color = [
                   1.0f32, 0.0f32, 0.0f32,
                   0.0f32, 1.0f32, 0.0f32,
                   0.0f32, 0.0f32, 1.0f32,
                   0.0f32, 0.0f32, 1.0f32,
                   1.0f32, 1.0f32, 1.0f32,
                   0.0f32, 0.0f32, 1.0f32,
                   0.0f32, 0.0f32, 1.0f32,
                   0.0f32, 1.0f32, 0.0f32,
                   1.0f32, 0.0f32, 0.0f32,
                ];

            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[1]);
            let color_u8 = color.to_u8_slice();
            gl.BufferData(
                gl33::GL_ARRAY_BUFFER,
                color_u8.len() as isize,
                color_u8.as_ptr().cast(),
                gl33::GL_STATIC_DRAW,
            );
            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, 0);

            gl.BindBuffer(gl33::GL_ELEMENT_ARRAY_BUFFER, df.vbo[2]);

            let indices = [
                2_u16,
                0_u16,
                1_u16,
                u16::MAX,
                3_u16,
                1_u16,
                8_u16,
                u16::MAX,
                6_u16,
                7_u16,
                8_u16,
                u16::MAX,
                5_u16,
                0_u16,
                7_u16,
                u16::MAX,
                4_u16,
                2_u16,
                3_u16,
                6_u16,
                5_u16,
                2_u16,
            ];
            let indices_u8 = indices.to_u8_slice();
            INDICES_NUM = indices.len() as i32;

            gl.BufferData(
                gl33::GL_ELEMENT_ARRAY_BUFFER,
                indices_u8.len() as isize,
                indices_u8.as_ptr().cast(),
                gl33::GL_STATIC_DRAW,
            );
            gl.BindBuffer(gl33::GL_ELEMENT_ARRAY_BUFFER, 0);

            // Create VAO to handle VBO
            let mut vao = 0_u32;
            gl.GenVertexArrays(1, &mut vao as *mut u32);

            gl.BindVertexArray(vao);

            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[0]);
            gl.EnableVertexAttribArray(0);
            gl.VertexAttribPointer(0, 3, gl33::GL_FLOAT, 0, 0, 0 as *const std::ffi::c_void);

            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[1]);
            gl.EnableVertexAttribArray(1);
            gl.VertexAttribPointer(1, 3, gl33::GL_FLOAT, 0, 0, 0 as *const std::ffi::c_void);

            gl.BindBuffer(gl33::GL_ELEMENT_ARRAY_BUFFER, df.vbo[2]);

            gl.BindVertexArray(0);
            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, 0);
            gl.BindBuffer(gl33::GL_ELEMENT_ARRAY_BUFFER, 0);

            df.vao = Some(vao);
        }
    }

    unsafe {
        let gl = df.gl.gl();

        gl.Viewport(0, 0, df.width, df.height);
        gl.ClearColor(0.07_f32, 0.13_f32, 0.17_f32, 1.0_f32);
        gl.Clear(gl33::GL_COLOR_BUFFER_BIT);

        // Enable VBO and corresponding through VAO
        gl.BindVertexArray(df.vao.unwrap());

        // GL_PRIMITIVE_RESTART_INDEX is used to retrieve the current primitive restart index
        // value.
        //
        // Following will return "0" for the first time and "0xffff = u16::MAX" which is set by
        // using gl.PrimitiveRestartIndex() below for the second time and later.
        //
        // Note:"0x1234" will never be printed.
        let mut restart_index = 0x1234;
        gl.GetIntegerv(
            gl33::GL_PRIMITIVE_RESTART_INDEX,
            &mut restart_index as *mut i32,
        );

        jinfo!(RestartIndex = format!("{:x}", restart_index));

        // Specify primitive restart index value
        gl.PrimitiveRestartIndex(u16::MAX as u32);

        gl.PointSize(10.0f32);

        // Enable primitive restart
        //
        // Note that gl33 does not provide enum corresponding to
        // "GL_PRIMITIVE_RESTART_FIXED_INDEX" which uses u16::MAX/u8::MAX for primitive restart
        // index value. So we have to specify restart index value with PrimitiveRestartIndex()
        // and enable restart by using GL_PRIMITIVE_RESTART
        gl.Enable(gl33::GL_PRIMITIVE_RESTART);
        gl.DrawElements(
            gl33::GL_TRIANGLE_FAN,
            INDICES_NUM,
            gl33::GL_UNSIGNED_SHORT,
            0 as *const std::ffi::c_void,
        );
        gl.Disable(gl33::GL_PRIMITIVE_RESTART);

        gl.BindVertexArray(0);

        gl.Flush();
    }

    Ok(())
}
