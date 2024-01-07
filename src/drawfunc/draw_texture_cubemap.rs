use super::{elapsed_milliseconds, DrawContext, DrawFunc};
use error_stack::{Report, Result};
use jlogger_tracing::jdebug;
use libogl::error::OglError;
use libogl::texture2d::Texture2DFilter;

pub fn draw_texture_cubemap(df: &mut DrawContext) -> Result<(), OglError> {
    if !df.initialized || df.draw_func != DrawFunc::DrawTexture3 {
        let v_src = r#"
                #version 300 es
                layout(location = 0) in vec3 vPosition;
                layout(location = 1) in vec3 vTexCoord;

                uniform mat4 u_mvpMatrix;
                uniform mat4 u_view;

                out vec3 vTextureCoord;

                void main()
                {
                    gl_Position = u_mvpMatrix * u_view * vec4(vPosition, 1.0f);
                    vTextureCoord= vTexCoord;
                }
        "#;

        let f_src = r#"
                #version 300 es
                precision mediump float;

                out vec4 fragColor;
                in vec3 vTextureCoord;

                uniform samplerCube u_Texture;

                void main()
                {
                    fragColor= texture(u_Texture, vTextureCoord);
                }
        "#;

        df.gl.build(Some(v_src), Some(f_src))?;

        df.initialized = true;
        df.draw_func = DrawFunc::DrawTexture3;

        unsafe {
            let gl = df.gl.gl();
            let program = df.gl.program().unwrap();
            gl.UseProgram(program);

            let images = vec![
                "right.jpg",
                "left.jpg",
                "top.jpg",
                "bottom.jpg",
                "back.jpg",
                "front.jpg",
            ]
            .into_iter()
            .map(|a| {
                format!(
                    "{}/{a}",
                    std::env::var("OGL_IMAGES").unwrap_or(".".to_owned())
                )
            })
            .collect::<Vec<String>>();

            df.texture_cubemap[0].create_from_file(
                images.iter().map(|a| a.as_str()).collect::<Vec<&str>>(),
                gl,
                Texture2DFilter::Linear,
            )?;

            jdebug!("texture_cubemap: {}", df.texture_cubemap[0]);

            df.locations[0] = df
                .location("u_mvpMatrix")
                .ok_or(Report::new(OglError::Unexpected))?;

            df.locations[1] = df
                .location("u_view")
                .ok_or(Report::new(OglError::Unexpected))?;

            df.locations[2] = df
                .location("u_Texture")
                .ok_or(Report::new(OglError::Unexpected))?;

            // Create VBO for vertex and color
            gl.GenBuffers(3, &mut df.vbo as *mut u32);

            #[rustfmt::skip]
            let vertices = [
                //          x           y           z
                glam::vec3(-0.5f32, -0.5f32, -0.5f32),  //v0
                glam::vec3(-0.5f32, -0.5f32,  0.5f32),  //v1
                glam::vec3( 0.5f32, -0.5f32,  0.5f32),  //v2
                glam::vec3( 0.5f32, -0.5f32, -0.5f32),  //v3

                glam::vec3(-0.5f32,  0.5f32, -0.5f32),  //v4
                glam::vec3(-0.5f32,  0.5f32,  0.5f32),  //v5
                glam::vec3( 0.5f32,  0.5f32,  0.5f32),  //v6
                glam::vec3( 0.5f32,  0.5f32, -0.5f32),  //v7
            ];

            #[rustfmt::skip]
            let indices = [
                0_u16,   2_u16,  1_u16,
                0_u16,   3_u16,  2_u16,

                4_u16,   5_u16,  6_u16,
                4_u16,   6_u16,  7_u16,

                0_u16,   4_u16,  7_u16,
                0_u16,   7_u16,  3_u16,

                1_u16,   6_u16,  5_u16,
                1_u16,   2_u16,  6_u16,

                0_u16,   1_u16,  5_u16,
                0_u16,   5_u16,  4_u16,

                3_u16,   7_u16,  6_u16,
                3_u16,   6_u16,  2_u16
            ];
            df.vertex_number = indices.len() as u32;

            let vertices_f32: Vec<f32> = vertices.iter().map(|a| a.to_array()).flatten().collect();
            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[0]);

            gl.BufferData(
                gl33::GL_ARRAY_BUFFER,
                (vertices_f32.len() * std::mem::size_of::<f32>()) as isize,
                vertices_f32.as_ptr().cast(),
                gl33::GL_STATIC_DRAW,
            );

            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[1]);
            let mut vertices_norm_f32: Vec<f32> = vec![];
            for i in indices.iter().map(|a| *a as usize) {
                vertices_norm_f32 = vertices_norm_f32
                    .into_iter()
                    .chain(vertices[i].normalize().to_array().into_iter())
                    .collect();
            }

            //            jdebug!(
            //                vertices_norm_f32 = format!("vertices_norm_f32: {:?}", vertices_norm_f32),
            //                len = vertices_norm_f32.len()
            //            );

            gl.BufferData(
                gl33::GL_ARRAY_BUFFER,
                (vertices_norm_f32.len() * std::mem::size_of::<f32>()) as isize,
                vertices_norm_f32.as_ptr().cast(),
                gl33::GL_STATIC_DRAW,
            );

            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, 0);

            gl.BindBuffer(gl33::GL_ELEMENT_ARRAY_BUFFER, df.vbo[2]);

            gl.BufferData(
                gl33::GL_ELEMENT_ARRAY_BUFFER,
                (indices.len() * std::mem::size_of::<u16>()) as isize,
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
            gl.VertexAttribPointer(0, 3, gl33::GL_FLOAT, 0, 0, 0 as *const std::ffi::c_void);

            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[1]);
            gl.EnableVertexAttribArray(1);
            gl.VertexAttribPointer(1, 3, gl33::GL_FLOAT, 0, 0, 0 as *const std::ffi::c_void);

            // Bind texture cube map
            df.texture_cubemap[0].bind(gl, 0, df.locations[2])?;

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

        // Scale matrix
        let scale = glam::Mat4::from_scale(glam::vec3(1.0f32, 1.0f32, 1.0f32));

        // Rotate matrix
        let angle = (elapsed_milliseconds() / 16 % u32::MAX as u128) as f32;
        let rotate_x = glam::Mat4::from_quat(glam::Quat::from_rotation_x(angle.to_radians()));

        let rotate_y = glam::Mat4::from_quat(glam::Quat::from_rotation_y(angle.to_radians()));

        let rotate_z = glam::Mat4::from_quat(glam::Quat::from_rotation_z(angle.to_radians()));

        // Translate matrix
        let translate = glam::Mat4::from_translation(glam::Vec3::new(0.0f32, 0.0f32, 5.0f32));
        //jdebug!(translate = format!("{:?}", translate));

        // Perspective matrix
        let aspect = df.width as f32 / df.height as f32;
        let fov = 45.0f32.to_radians();
        let near = 0.1f32;
        let far = 20.0f32;
        //jdebug!(near = near, far = far, fov = fov);
        let perspective = glam::Mat4::perspective_lh(fov, aspect, near, far);

        // Cube is operated from right to left
        //  1. rotate z
        //  2. rotate y
        //  3. rotate x
        //  4. translate
        //  5. scale
        //  6. perspective
        let mvp = perspective * scale * translate * rotate_x * rotate_y * rotate_z;

        //jdebug!(mvp = format!("{:?}", mvp));
        gl.UniformMatrix4fv(df.locations[0], 1, 0, mvp.as_ref().as_ptr());

        // Camera view matrix
        let eye = glam::Vec3::new(0.0f32, 0.0f32, 0.0f32);
        let target = glam::Vec3::new(0.5f32, 0.5f32, 0.0f32);
        let up = glam::vec3(0.0f32, 0.1f32, 0.0f32);
        let view = glam::Mat4::look_to_rh(eye, target, up);
        //let view = glam::Mat4::IDENTITY;
        gl.UniformMatrix4fv(df.locations[1], 1, 0, view.as_ref().as_ptr().cast());

        //gl.Enable(gl33::GL_CULL_FACE);
        //gl.FrontFace(gl33::GL_CCW);
        //gl.Enable(gl33::GL_DEPTH_TEST);
        //gl.DepthFunc(gl33::GL_LESS);
        //gl.CullFace(gl33::GL_FRONT);
        //gl.DepthFunc(gl33::GL_LEQUAL);

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
