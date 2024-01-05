use super::{DrawContext, DrawFunc};
use error_stack::Result;
use jlogger_tracing::jdebug;
use libogl::error::OglError;
use libogl::texture2d::Texture2DFilter;
use libogl::VertexOps;

pub fn draw_texture(df: &mut DrawContext) -> Result<(), OglError> {
    if !df.initialized || df.draw_func != DrawFunc::DrawTexture {
        let v_src = r#"
                #version 300 es
                layout(location = 0) in vec4 vPosition;
                layout(location = 1) in vec2 vTexCoord;

                out vec2 vTextureCoord;

                void main()
                {
                    gl_Position = vPosition;
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
        df.draw_func = DrawFunc::DrawTexture;

        unsafe {
            let gl = df.gl.gl();
            let program = df.gl.program().unwrap();

            let data = include_bytes!("../../doc/sample.png");
            df.texture[0].create_from_buffer(data, gl, Texture2DFilter::Linear)?;
            jdebug!("texture: {}", df.texture[0]);

            gl.UseProgram(program);

            #[rustfmt::skip]
            let vertices = [
                //x        y                     z          
                0.0f32,    f32::sqrt(0.5), 0.0f32,
               -0.5f32,   -0.5f32,               0.0f32,
                0.5f32,   -0.5f32,               0.0f32,
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

            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[1]);
            // Texture coordinates falls into the range [0, 1].
            #[rustfmt::skip]
            let texture_vertex = [
              //s       t
                0.5f32, 1.0f32,
                0.0f32, 0.0f32,
                1.0f32, 0.0f32,
            ];

            let texture_vertex_u8 = texture_vertex.to_u8_slice();
            gl.BufferData(
                gl33::GL_ARRAY_BUFFER,
                texture_vertex_u8.len() as isize,
                texture_vertex_u8.as_ptr().cast(),
                gl33::GL_STATIC_DRAW,
            );
            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, 0);

            gl.BindBuffer(gl33::GL_ELEMENT_ARRAY_BUFFER, df.vbo[2]);
            let indices = [0_u16, 1_u16, 2_u16];
            let indices_u8 = indices.to_u8_slice();

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
            gl.VertexAttribPointer(1, 2, gl33::GL_FLOAT, 0, 0, 0 as *const std::ffi::c_void);

            // Bind texture
            let location = df.location("u_Texture").unwrap();
            df.texture[0].bind(gl, 0, location)?;

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
            gl33::GL_TRIANGLES,
            3,
            gl33::GL_UNSIGNED_SHORT,
            0 as *const std::ffi::c_void,
        );

        gl.BindVertexArray(0);
        gl.Flush();
    }

    Ok(())
}
