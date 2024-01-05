use super::{elapsed_milliseconds, DrawContext, DrawFunc};
use error_stack::{Report, Result};
use jlogger_tracing::jdebug;
use libogl::error::OglError;
use libogl::matrix::{OglMatrix, RotateDirection};
use libogl::texture2d::Texture2DFilter;
use libogl::VertexOps;

pub fn draw_texture_mipmapping(df: &mut DrawContext) -> Result<(), OglError> {
    if !df.initialized || df.draw_func != DrawFunc::DrawTextureMipMapping {
        let v_src = r#"
                #version 300 es
                layout(location = 0) in vec4 vPosition;
                layout(location = 1) in vec2 vTexCoord;
                uniform mat4 u_mvpMatrix;

                out vec2 vTextureCoord;

                void main()
                {
                    gl_Position = u_mvpMatrix * vPosition;
                    vTextureCoord= vTexCoord;

                }
        "#;

        let f_src = r#"
                #version 300 es
                precision mediump float;
                out vec4 fragColor;

                in vec2 vTextureCoord;
                uniform sampler2D u_Texture;

                void main()
                {
                    fragColor= texture2D(u_Texture, vTextureCoord);
                }
        "#;

        df.gl.build(Some(v_src), Some(f_src))?;

        df.initialized = true;
        df.draw_func = DrawFunc::DrawTextureMipMapping;

        unsafe {
            let gl = df.gl.gl();
            let program = df.gl.program().unwrap();

            let data = include_bytes!("../../doc/sample2.png");
            df.texture[0].create_from_buffer(data, gl, Texture2DFilter::NearestMiMapNearest)?;
            jdebug!("texture: {}", df.texture[0]);

            gl.UseProgram(program);

            df.locations[0] = df
                .location("u_mvpMatrix")
                .ok_or(Report::new(OglError::Unexpected))?;

            // Create VBO for vertex and color
            gl.GenBuffers(2, &mut df.vbo as *mut u32);

            #[rustfmt::skip]
            let vertices = [
                //x        y      z             s       t
                -0.5f32, -0.5f32, -0.5f32,      1.0f32, 0.0f32,     //v0
                -0.5f32, -0.5f32,  0.5f32,      0.0f32, 0.0f32,     //v1
                 0.5f32, -0.5f32,  0.5f32,      0.0f32, 1.0f32,     //v2
                 0.5f32, -0.5f32, -0.5f32,      1.0f32, 1.0f32,     //v3

                -0.5f32,  0.5f32, -0.5f32,      0.0f32, 0.0f32,     //v4
                -0.5f32,  0.5f32,  0.5f32,      1.0f32, 0.0f32,     //v5
                 0.5f32,  0.5f32,  0.5f32,      1.0f32, 1.0f32,     //v6
                 0.5f32,  0.5f32, -0.5f32,      0.0f32, 1.0f32,     //v7

                -0.5f32, -0.5f32, -0.5f32,      1.0f32, 0.0f32,     //v8  = v0
                -0.5f32,  0.5f32, -0.5f32,      1.0f32, 1.0f32,     //v9  = v4
                 0.5f32,  0.5f32, -0.5f32,      0.0f32, 1.0f32,     //v10 = v7
                 0.5f32, -0.5f32, -0.5f32,      0.0f32, 0.0f32,     //v11 = v3

                -0.5f32, -0.5f32,  0.5f32,      0.0f32, 0.0f32,     //v12 = v1
                -0.5f32,  0.5f32,  0.5f32,      0.0f32, 1.0f32,     //v13 = v5
                 0.5f32,  0.5f32,  0.5f32,      1.0f32, 1.0f32,     //v14 = v6
                 0.5f32, -0.5f32,  0.5f32,      1.0f32, 0.0f32,     //v15 = v2

                -0.5f32, -0.5f32, -0.5f32,      0.0f32, 0.0f32,     //v16 = v0
                -0.5f32, -0.5f32,  0.5f32,      1.0f32, 0.0f32,     //v17 = v1
                -0.5f32,  0.5f32,  0.5f32,      1.0f32, 1.0f32,     //v18 = v5
                -0.5f32,  0.5f32, -0.5f32,      0.0f32, 1.0f32,     //v19 = v4

                 0.5f32, -0.5f32, -0.5f32,      1.0f32, 0.0f32,     //v20 = v3
                 0.5f32, -0.5f32,  0.5f32,      0.0f32, 0.0f32,     //v21 = v2
                 0.5f32,  0.5f32,  0.5f32,      0.0f32, 1.0f32,     //v22 = v6
                 0.5f32,  0.5f32, -0.5f32,      1.0f32, 1.0f32,     //v23 = v7
            ];

            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[0]);

            let vertices_u8 = vertices.to_u8_slice();
            gl.BufferData(
                gl33::GL_ARRAY_BUFFER,
                vertices_u8.len() as isize,
                vertices_u8.as_ptr().cast(),
                gl33::GL_STATIC_DRAW,
            );
            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, 0);

            gl.BindBuffer(gl33::GL_ELEMENT_ARRAY_BUFFER, df.vbo[1]);

            #[rustfmt::skip]
            let indices = [
                0_u16,   2_u16,  1_u16,
                0_u16,   3_u16,  2_u16,

                4_u16,   5_u16,  6_u16,
                4_u16,   6_u16,  7_u16,

                8_u16,   9_u16,  10_u16,
                8_u16,  10_u16,  11_u16,

               12_u16,  14_u16, 13_u16,
               12_u16,  15_u16, 14_u16,

               16_u16,  17_u16, 18_u16,
               16_u16,  18_u16, 19_u16,

               20_u16,  23_u16, 22_u16,
               20_u16,  22_u16, 21_u16
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
            let stride = std::mem::size_of::<f32>() * 5;
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

            offset = std::mem::size_of::<f32>() * 3;

            gl.EnableVertexAttribArray(1);
            gl.VertexAttribPointer(
                1,
                2,
                gl33::GL_FLOAT,
                0,
                stride as i32,
                offset as *const std::ffi::c_void,
            );

            // Bind texture
            let location = df.location("u_Texture").unwrap();
            df.texture[0].bind(gl, 0, location)?;

            gl.BindBuffer(gl33::GL_ELEMENT_ARRAY_BUFFER, df.vbo[1]);

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

        let aspect = df.width as f32 / df.height as f32;
        let mut mvp = OglMatrix::new(None);
        // Scale the cube
        mvp.scale(1.5f32, 1.5f32, 1.5f32)?;
        jdebug!("{}", format!("scaled mvp:\n{}", mvp));

        // Rotate the cube
        let angle = (elapsed_milliseconds() / 16 % u32::MAX as u128) as f32;
        //let angle = 135.0f32;
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
