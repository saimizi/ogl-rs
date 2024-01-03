use super::elapsed_milliseconds;
use super::{DrawContext, DrawFunc};
use error_stack::Result;
use jlogger_tracing::jdebug;
use libogl::error::OglError;
use libogl::matrix::OglMatrix;
use libogl::matrix::RotateDirection;
use libogl::VertexOps;

pub fn draw_model_view_projection(df: &mut DrawContext) -> Result<(), OglError> {
    if !df.initialized || df.draw_func != DrawFunc::DrawVaoVertexColorElement2 {
        let v_src = r#"
                #version 300 es

                uniform mat4 u_mvpMatrix;
                layout(location = 0) in vec4 vPosition;
                layout(location = 1) in vec4 vColor;

                out vec4 vColorVec;

                void main()
                {
                   gl_Position = u_mvpMatrix * vPosition;
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
            let program = df.gl.program().unwrap();

            gl.UseProgram(program);

            // Store the location of "u_mvpMatrix" to df->locations[0];
            let name = std::ffi::CString::new("u_mvpMatrix").unwrap();
            df.locations[0] = gl.GetUniformLocation(program, name.as_ptr().cast());

            #[rustfmt::skip]
            let vertices = [
                -0.5f32, -0.5f32, -0.5f32, //v0
                -0.5f32, -0.5f32,  0.5f32, //v1
                 0.5f32, -0.5f32,  0.5f32, //v2
                 0.5f32, -0.5f32, -0.5f32, //v3
                -0.5f32,  0.5f32, -0.5f32, //v4
                -0.5f32,  0.5f32,  0.5f32, //v5
                 0.5f32,  0.5f32,  0.5f32, //v6
                 0.5f32,  0.5f32, -0.5f32, //v7
                -0.5f32, -0.5f32, -0.5f32, //v8
                -0.5f32,  0.5f32, -0.5f32, //v9
                 0.5f32,  0.5f32, -0.5f32, //v10
                 0.5f32, -0.5f32, -0.5f32, //v11
                -0.5f32, -0.5f32,  0.5f32, //v12
                -0.5f32,  0.5f32,  0.5f32, //v13
                 0.5f32,  0.5f32,  0.5f32, //v14
                 0.5f32, -0.5f32,  0.5f32, //v15
                -0.5f32, -0.5f32, -0.5f32, //v16
                -0.5f32, -0.5f32,  0.5f32, //v17
                -0.5f32,  0.5f32,  0.5f32, //v18
                -0.5f32,  0.5f32, -0.5f32, //v19
                 0.5f32, -0.5f32, -0.5f32, //v20
                 0.5f32, -0.5f32,  0.5f32, //v21
                 0.5f32,  0.5f32,  0.5f32, //v22
                 0.5f32,  0.5f32, -0.5f32, //v23
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
               1.0f32, 0.0f32, 0.0f32,  //v0
               0.0f32, 1.0f32, 0.0f32,  //v1
               0.0f32, 1.0f32, 0.0f32,  //v2
               1.0f32, 1.0f32, 1.0f32,  //v3
               1.0f32, 0.0f32, 0.0f32,  //v4
               0.0f32, 1.0f32, 0.0f32,  //v5
               0.0f32, 0.0f32, 1.0f32,  //v6
               1.0f32, 1.0f32, 1.0f32,  //v7
               1.0f32, 0.0f32, 0.0f32,  //v8
               0.0f32, 1.0f32, 0.0f32,  //v9
               0.0f32, 0.0f32, 1.0f32,  //v10
               1.0f32, 1.0f32, 1.0f32,  //v11
               1.0f32, 0.0f32, 0.0f32,  //v12
               0.0f32, 1.0f32, 0.0f32,  //v13
               0.0f32, 0.0f32, 1.0f32,  //v14
               1.0f32, 1.0f32, 1.0f32,  //v15
               1.0f32, 0.0f32, 0.0f32,  //v16
               0.0f32, 1.0f32, 0.0f32,  //v17
               0.0f32, 0.0f32, 1.0f32,  //v18
               1.0f32, 1.0f32, 1.0f32,  //v19
               1.0f32, 0.0f32, 0.0f32,  //v20
               0.0f32, 1.0f32, 0.0f32,  //v21
               0.0f32, 0.0f32, 1.0f32,  //v22
               1.0f32, 1.0f32, 1.0f32,  //v23
            ];

            //            let mut color = vec![];
            //            for _i in 0..24 {
            //                color.push(1.0f32);
            //                color.push(0.0f32);
            //                color.push(0.0f32);
            //            }

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

            #[rustfmt::skip]
            let indices = [
                0_u16,   2_u16,  1_u16,
                0_u16,   3_u16,  2_u16,
                4_u16,   5_u16,  6_u16,
                4_u16,   6_u16,  7_u16,
                8_u16,   9_u16, 10_u16,
                8_u16,  10_u16, 11_u16,
                12_u16, 15_u16, 14_u16,
                12_u16, 14_u16, 13_u16,
                16_u16, 17_u16, 18_u16,
                16_u16, 18_u16, 19_u16,
                20_u16, 23_u16, 22_u16,
                20_u16, 22_u16, 21_u16
            ];

            let indices_u8 = indices.to_u8_slice();
            df.vertex_number = indices.len() as u32;

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

        // Enable VAO
        gl.BindVertexArray(df.vao.unwrap());
        let aspect = df.width as f32 / df.height as f32;

        let mut mvp = OglMatrix::new(None);

        // Scale the cube
        mvp.scale(1.5f32, 1.5f32, 1.5f32)?;
        jdebug!("{}", format!("scaled mvp:\n{}", mvp));

        // Rotate the cube
        let angle = (elapsed_milliseconds() / 16 % u32::MAX as u128) as f32;
        jdebug!(angle = angle);
        mvp.rotate(angle, RotateDirection::AxisX)?;
        mvp.rotate(angle, RotateDirection::AxisY)?;
        mvp.rotate(angle, RotateDirection::AxisZ)?;
        mvp.rotate(angle, RotateDirection::AxisXYZ(1.0f32, 0.0f32, 1.0f32))?;
        jdebug!("{}", format!("rotated mvp:\n{}", mvp));

        // Move the cube
        mvp.translate(0.0f32, 0.0f32, 5.0f32)?;
        jdebug!("{}", format!("translated mvp:\n{}", mvp));

        // Set perspective
        mvp.perspective(60.0f32, aspect, 1.0f32, 20.0f32)?;
        jdebug!("{}", format!("perspective mvp:\n{}", mvp));

        gl.UniformMatrix4fv(df.locations[0], 1, 1, mvp.as_ptr());

        gl.Enable(gl33::GL_CULL_FACE);
        gl.FrontFace(gl33::GL_CCW);
        gl.CullFace(gl33::GL_FRONT);

        jdebug!(vertices = df.vertex_number);
        gl.DrawElements(
            gl33::GL_TRIANGLES,
            df.vertex_number as i32,
            gl33::GL_UNSIGNED_SHORT,
            0 as *const std::ffi::c_void,
        );

        gl.BindVertexArray(0);
        gl.Flush();
    }

    Ok(())
}
