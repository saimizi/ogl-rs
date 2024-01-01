use super::{DrawContext, DrawFunc, VertexOps};
use error_stack::Result;
use libogl::error::OglError;

pub fn draw_triangle_strip(df: &mut DrawContext) -> Result<(), OglError> {
    static mut VERTICES_NUMBER: i32 = 0;
    if !df.initialized || df.draw_func != DrawFunc::DrawTriangleStrip {
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
        df.draw_func = DrawFunc::DrawTriangleStrip;

        unsafe {
            let gl = df.gl.gl();
            gl.UseProgram(df.gl.program().unwrap());

            gl.GenBuffers(3, &mut df.vbo as *mut u32);

            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[0]);
            // w decided the clip range,
            //      -w <= x <= w
            //      -w <= y <= w
            //      -w <= z <= w
            #[rustfmt::skip]
            let vertices = [
           //   x           y       z       w
               -0.0f32,   -0.5f32, 0.0f32,  2.0f32,  // v0
               -0.0f32,   -0.1f32, 0.0f32,  2.0f32,  // v1
                0.3f32,   -0.5f32, 0.0f32,  2.0f32,  // v2
                0.5f32,   -0.1f32, 0.0f32,  2.0f32,  // v3
                0.8f32,   -0.5f32, 0.0f32,  2.0f32,  // v4

                0.0f32,    0.0f32, 0.0f32,  1.0f32,  // v5
                0.0f32,    0.0f32, 0.0f32,  1.0f32,  // v6
                0.0f32,    0.0f32, 0.0f32,  1.0f32,  // v7

               -0.8f32,    0.2f32, 0.0f32,  0.8f32,  // v8
               -0.5f32,    0.6f32, 0.0f32,  0.8f32,  // v9
               -0.2f32,    0.2f32, 0.0f32,  0.8f32,  // v10
                0.0f32,    0.7f32, 0.0f32,  0.8f32,  // v11
            ];

            // Create VBO for vertex and color

            let vertices_u8 = vertices.to_u8_slice();
            gl.BufferData(
                gl33::GL_ARRAY_BUFFER,
                vertices_u8.len() as isize,
                vertices_u8.as_ptr().cast(),
                gl33::GL_STATIC_DRAW,
            );

            #[rustfmt::skip]
            let color = [
               1.0f32, 0.0f32, 0.0f32,      // v0
               0.0f32, 1.0f32, 0.0f32,      // v1
               0.0f32, 0.0f32, 1.0f32,      // v2
               1.0f32, 1.0f32, 1.0f32,      // v3

               1.0f32, 0.0f32, 0.0f32,      // v4
               0.0f32, 1.0f32, 0.0f32,      // v5
               0.0f32, 0.0f32, 1.0f32,      // v6
               1.0f32, 1.0f32, 1.0f32,      // v7

               1.0f32, 0.0f32, 0.0f32,      // v8
               0.0f32, 1.0f32, 0.0f32,      // v9
               0.0f32, 0.0f32, 1.0f32,      // v10
               1.0f32, 1.0f32, 1.0f32,      // v11
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

            // We draw two separated triangle strips (v0, v1, v2, v3 v4) and (v8, v9, v10, v11)with
            // one DrawElements() call by using degenerate triangles.
            //
            // 1st triangle strip:      (0,1,2), (1,2,3), (2,3,4)
            // degenerate triangles:   (3,4,4), (4,4,8), (4,8,8), (8,8,9)
            // 2nd triangle strip:      (8,9,10), (9,10,11)
            //
            // Since GPU will detect and reject degenerate triangles, we can transit drawing first
            // strip to the second by using them.
            let indices = [
                0_u8, 1_u8, 2_u8, 3_u8, 4_u8, 4_u8, 8_u8, 8_u8, 9_u8, 10_u8, 11_u8,
            ];
            //let indices = [8_u8, 9_u8, 10_u8, 11_u8, 11_u8, 0_u8, 0_u8, 1_u8, 2_u8, 3_u8, 4_u8];
            VERTICES_NUMBER = indices.len() as i32;
            gl.BufferData(
                gl33::GL_ELEMENT_ARRAY_BUFFER,
                indices.len() as isize,
                indices.as_ptr().cast(),
                gl33::GL_STATIC_DRAW,
            );
            gl.BindBuffer(gl33::GL_ELEMENT_ARRAY_BUFFER, 0);

            // Create VAO to handle VBO
            let mut vao = 0_u32;
            gl.GenVertexArrays(1, &mut vao as *mut u32);
            gl.BindVertexArray(vao);

            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[0]);
            gl.EnableVertexAttribArray(0);
            gl.VertexAttribPointer(0, 4, gl33::GL_FLOAT, 0, 0, 0 as *const std::ffi::c_void);

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

        gl.DrawElements(
            gl33::GL_TRIANGLE_STRIP,
            VERTICES_NUMBER,
            gl33::GL_UNSIGNED_BYTE,
            0 as *const std::ffi::c_void,
        );

        gl.BindVertexArray(0);
        gl.Flush();
    }

    Ok(())
}
