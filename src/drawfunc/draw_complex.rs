use super::{DrawContext, DrawFunc, VertexOps};
use error_stack::{Report, Result};
use libogl::error::OglError;
use std::f32::consts::PI;

pub fn draw_complex(df: &mut DrawContext) -> Result<(), OglError> {
    if !df.initialized || df.draw_func != DrawFunc::DrawComplex {
        let v_src = include_str!("../../es300/sample.vert");
        let f_src = include_str!("../../es300/sample.frag");
        df.gl.build(Some(v_src), Some(f_src))?;

        df.initialized = true;
        df.draw_func = DrawFunc::DrawComplex;
    }

    let gl = df.gl.gl();
    let program = df.gl.program().ok_or(Report::new(OglError::InvalidData))?;

    if df.w == 0 || df.h == 0 {
        df.turn_small = false;
    }

    if df.w >= df.width || df.h >= df.height {
        df.turn_small = true;
    }

    if df.turn_small {
        df.w -= 2;
        df.h -= 2;
    } else {
        df.w += 4;
        df.h += 4;
    }

    unsafe {
        gl.Viewport(df.w / 2, df.h / 2, df.w, df.h);
        gl.ClearColor(0f32, 0f32, 0f32, 0.2f32);
        gl.Clear(gl33::GL_COLOR_BUFFER_BIT);
        gl.UseProgram(df.gl.program().unwrap());

        {
            let name = std::ffi::CString::new("u_Color").unwrap();
            let location = gl.GetUniformLocation(program, name.as_ptr().cast());

            static mut C: [f32; 3] = [0.01, 0.8, 0.5];
            static mut D: [f32; 3] = [0.0001, 0.0001, 0.0001];

            for i in 0..C.len() {
                if C[i] <= 0.0 || C[i] >= 1.0 {
                    D[i] *= -1.0;
                }

                C[i] += D[i];
            }

            gl.Uniform4f(location, C[0], C[1], C[2], 1.0f32);
        }

        if df.vbo[0] == 0 || df.vbo[1] == 0 {
            gl.GenBuffers(2, &mut df.vbo as *mut u32);

            #[rustfmt::skip]
                let vertices = [
                    -0.9f32, -0.7f32,
                    -0.9f32,  0.7f32,
                    -0.2f32,  0.0f32,
                    -0.7f32,  0.9f32,
                     0.7f32,  0.9f32,
                     0.0f32,  0.2f32,
                     0.2f32,  0.0f32,
                     0.9f32,  0.7f32,
                     0.9f32, -0.7f32,
                     0.0f32, -0.2f32,
                     0.7f32, -0.9f32,
                    -0.7f32, -0.9f32,
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

            let size = (core::mem::size_of::<f32>() * 2 * 362) as isize;
            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[1]);
            gl.BufferData(
                gl33::GL_ARRAY_BUFFER,
                size,
                core::ptr::null(),
                gl33::GL_STATIC_DRAW,
            );

            let buffer_p = gl.MapBufferRange(gl33::GL_ARRAY_BUFFER, 0, size, gl33::GL_MAP_WRITE_BIT)
                as *mut f32;

            let mut j = 2;
            let r = 0.2f32;
            *buffer_p.add(0) = 0f32;
            *buffer_p.add(1) = 0f32;
            for i in 0..360 {
                *buffer_p.add(j) = r * f32::cos(i as f32 * PI / 180f32);
                j += 1;
                *buffer_p.add(j) = r * f32::sin(i as f32 * PI / 180f32);
                j += 1;
            }

            *buffer_p.add(722) = r * libm::cos(0f64) as f32;
            *buffer_p.add(723) = r * libm::sin(0f64) as f32;
            gl.UnmapBuffer(gl33::GL_ARRAY_BUFFER);

            gl.BindBuffer(gl33::GL_ARRAY_BUFFER, 0);
        }

        gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[0]);

        gl.VertexAttribPointer(0, 2, gl33::GL_FLOAT, 0, 0, 0 as *const std::ffi::c_void);
        gl.EnableVertexAttribArray(0);
        gl.DrawArrays(gl33::GL_TRIANGLES, 0, 12);
        gl.DisableVertexAttribArray(0);

        gl.BindBuffer(gl33::GL_ARRAY_BUFFER, df.vbo[1]);
        gl.EnableVertexAttribArray(0);
        gl.VertexAttribPointer(0, 2, gl33::GL_FLOAT, 0, 0, 0 as *const std::ffi::c_void);
        gl.DrawArrays(gl33::GL_TRIANGLE_FAN, 0, 362);
        gl.BindBuffer(gl33::GL_ARRAY_BUFFER, 0);
        gl.DisableVertexAttribArray(0);

        gl.Flush();
    }

    Ok(())
}
