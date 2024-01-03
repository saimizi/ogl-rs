use super::{elapsed_seconds, DrawContext, DrawFunc};
use error_stack::Result;
use libogl::error::OglError;
use libogl::VertexOps;
use rand::Rng;
use std::f32::consts::PI;

pub fn draw_provoking_vertex(df: &mut DrawContext) -> Result<(), OglError> {
    static mut VERTICES_NUM: usize = 0;
    {
        let mut v_src = r#"
                #version 300 es
                layout(location = 0) in vec4 vPosition;
                layout(location = 1) in vec4 vColor;

                flat out vec4 vColorVec;

                void main()
                {
                    gl_Position = vPosition;
                    vColorVec = vColor;

                }
        "#
        .to_owned();

        let mut f_src = r#"
                #version 300 es
                precision mediump float;
                out vec4 fragColor;

                flat in vec4 vColorVec;
                void main()
                {
                    fragColor = vColorVec ;
                }
        "#
        .to_owned();

        // Change "flat" to "smooth" interpolator very 3 seconds.
        if elapsed_seconds() % 3 == 0 {
            v_src = v_src.replace("flat", "");
            f_src = f_src.replace("flat", "");
        }

        df.gl.build(Some(&v_src), Some(&f_src))?;
        df.gl.gl().UseProgram(df.gl.program().unwrap());
    }

    if !df.initialized || df.draw_func != DrawFunc::DrawProvokingVertex {
        df.initialized = true;
        df.draw_func = DrawFunc::DrawProvokingVertex;

        unsafe {
            let gl = df.gl.gl();
            let mut vertices = vec![];
            let r = 0.5f32;

            vertices.push(0.0f32);
            vertices.push(0.0f32);
            vertices.push(0.0f32);

            for i in 0..=360 {
                let x = r * f32::cos(i as f32 * 1.0f32 / (2.0f32 * PI));
                let y = r * f32::sin(i as f32 * 1.0f32 / (2.0f32 * PI));
                let z = 0.0f32;

                vertices.push(x);
                vertices.push(y);
                vertices.push(z);
            }

            VERTICES_NUM = vertices.len() / 3;

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

            let mut color = vec![];
            let mut rng = rand::thread_rng();
            for _i in 0..VERTICES_NUM {
                let r = rng.gen_range(0.0f32..1.0f32);
                let g = rng.gen_range(0.0f32..1.0f32);
                let b = rng.gen_range(0.0f32..1.0f32);

                color.push(r);
                color.push(g);
                color.push(b);
            }
            let color_u8 = color.to_u8_slice();

            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[1]);
            gl.BufferData(
                gl33::GL_ARRAY_BUFFER,
                color_u8.len() as isize,
                color_u8.as_ptr().cast(),
                gl33::GL_STATIC_DRAW,
            );
            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, 0);

            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[0]);
            gl.EnableVertexAttribArray(0);
            gl.VertexAttribPointer(0, 3, gl33::GL_FLOAT, 0, 0, 0 as *const std::ffi::c_void);

            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[1]);
            gl.EnableVertexAttribArray(1);
            gl.VertexAttribPointer(1, 3, gl33::GL_FLOAT, 0, 0, 0 as *const std::ffi::c_void);

            gl.BindBuffer(gl33::GL_ELEMENT_ARRAY_BUFFER, df.vbo[2]);

            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, 0);
            gl.BindBuffer(gl33::GL_ELEMENT_ARRAY_BUFFER, 0);
        }
    }

    unsafe {
        let gl = df.gl.gl();

        gl.Viewport(0, 0, df.width, df.height);
        gl.ClearColor(0.07_f32, 0.13_f32, 0.17_f32, 1.0_f32);
        gl.Clear(gl33::GL_COLOR_BUFFER_BIT);

        let mut indices = vec![];
        for i in 0..VERTICES_NUM {
            indices.push(i as u16);
        }

        let indices_u8 = indices.to_u8_slice();

        gl.BindBuffer(gl33::GL_ELEMENT_ARRAY_BUFFER, df.vbo[2]);

        gl.BufferData(
            gl33::GL_ELEMENT_ARRAY_BUFFER,
            indices_u8.len() as isize,
            indices_u8.as_ptr().cast(),
            gl33::GL_STATIC_DRAW,
        );

        gl.DrawElements(
            gl33::GL_TRIANGLE_FAN,
            VERTICES_NUM as i32,
            gl33::GL_UNSIGNED_SHORT,
            0 as *const std::ffi::c_void,
        );
        gl.BindBuffer(gl33::GL_ELEMENT_ARRAY_BUFFER, 0);

        gl.Flush();
    }

    Ok(())
}
