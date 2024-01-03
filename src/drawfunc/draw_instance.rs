use super::{DrawContext, DrawFunc};
use error_stack::Result;
use libogl::error::OglError;
use libogl::VertexOps;
use std::f32::consts::PI;

pub fn draw_instance(df: &mut DrawContext) -> Result<(), OglError> {
    static mut VERTICES_NUMBER: usize = 0;

    if !df.initialized || df.draw_func != DrawFunc::DrawCircle {
        let v_src = r#"
                #version 300 es
                layout(location = 0) in vec4 vPosition;
                layout(location = 1) in vec4 vColor;
                layout(location = 2) in vec4 vOffset;

                out vec4 vColorVec;
                

                void main()
                {
                    gl_Position = vPosition + vOffset;
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
        df.draw_func = DrawFunc::DrawCircle;

        unsafe {
            let gl = df.gl.gl();
            gl.UseProgram(df.gl.program().unwrap());

            let mut vertices = vec![];

            vertices.push(0.05f32);
            vertices.push(0.05f32);
            vertices.push(0.0f32);

            let r = 0.2;

            for i in 0..=360 {
                let unit = (i as f32) * PI / 180f32;
                let x = r * f32::cos(unit);
                let y = r * f32::sin(unit);

                vertices.push(x);
                vertices.push(y);
                vertices.push(0f32);
            }

            VERTICES_NUMBER = vertices.len() / 3;

            // Create VBO for vertex and color
            gl.GenBuffers(4, &mut df.vbo as *mut u32);
            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[0]);

            let vertices_u8 = vertices.to_u8_slice();
            gl.BufferData(
                gl33::GL_ARRAY_BUFFER,
                vertices_u8.len() as isize,
                vertices_u8.as_ptr().cast(),
                gl33::GL_STATIC_DRAW,
            );
            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, 0);

            let mut color = vec![];

            for i in 0..VERTICES_NUMBER {
                let unit = (i as f32) * PI / 180f32;
                let rc = r * f32::cos(unit);
                let gc = r * f32::sin(unit);
                let bc = 0.3 * f32::sin(unit) * f32::sin(unit);

                if i == 0 {
                    color.push(0.9f32);
                    color.push(0.8f32);
                    color.push(0.7f32);
                } else {
                    color.push(rc);
                    color.push(gc);
                    color.push(bc);
                }
            }

            assert_eq!(vertices.len(), color.len());

            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[1]);

            let color_u8 = color.to_u8_slice();
            gl.BufferData(
                gl33::GL_ARRAY_BUFFER,
                color_u8.len() as isize,
                color_u8.as_ptr().cast(),
                gl33::GL_STATIC_DRAW,
            );

            // create VBO for offset
            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[2]);
            #[rustfmt::skip]
            let offset = vec![
                 1.0f32,  1.0f32, 0.0f32,
                -1.0f32,  1.0f32, 0.0f32,
                 0.0f32,  0.0f32, 0.0f32,
                 1.0f32, -1.0f32, 0.0f32,
                -1.0f32, -1.0f32, 0.0f32,
            ];
            let offset_u8 = offset.to_u8_slice();

            gl.BufferData(
                gl33::GL_ARRAY_BUFFER,
                offset_u8.len() as isize,
                offset_u8.as_ptr().cast(),
                gl33::GL_STATIC_DRAW,
            );

            // Create VBO for element indices.
            gl.BindBuffer(gl33::GL_ELEMENT_ARRAY_BUFFER, df.vbo[3]);

            let mut indices = vec![];
            for i in 0..VERTICES_NUMBER {
                indices.push(i as u16);
            }

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

            gl.EnableVertexAttribArray(0);
            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[0]);
            gl.VertexAttribPointer(0, 3, gl33::GL_FLOAT, 0, 0, core::ptr::null_mut());

            gl.EnableVertexAttribArray(1);
            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[1]);
            gl.VertexAttribPointer(1, 3, gl33::GL_FLOAT, 0, 0, core::ptr::null_mut());

            gl.EnableVertexAttribArray(2);
            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[2]);
            gl.VertexAttribPointer(2, 3, gl33::GL_FLOAT, 0, 0, core::ptr::null_mut());

            gl.BindBuffer(gl33::GL_ELEMENT_ARRAY_BUFFER, df.vbo[3]);

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
        let vao = df.vao.unwrap();
        gl.BindVertexArray(vao);

        gl.DrawElementsInstanced(
            gl33::GL_TRIANGLE_FAN,
            VERTICES_NUMBER as i32,
            gl33::GL_UNSIGNED_SHORT,
            core::ptr::null_mut(),
            5,
        );

        // Make "offset" attribute be loaded for instance instead of vertex.
        gl.VertexAttribDivisor(2, 1);

        gl.BindVertexArray(0);
        gl.Flush();
    }
    Ok(())
}
