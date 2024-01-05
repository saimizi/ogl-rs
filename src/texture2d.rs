use super::error::OglError;
use error_stack::{Report, Result};
use stb_image::stb_image::*;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::Read;
use std::slice;

pub enum Texture2DFilter {
    Nearest,
    Linear,
    NearestMiMapNearest,
    NearestMiMapLinear,
    LinearMiMapNearest,
    LinearMiMapLinear,
}

#[derive(Default, Debug, Clone)]
pub struct Texture2D {
    id: u32,
    data: Vec<u8>,
    width: i32,
    height: i32,
    bpp: i32,
}

impl Texture2D {
    pub unsafe fn create_from_file(
        &mut self,
        file_name: &str,
        gl: &gl33::GlFns,
        filter: Texture2DFilter,
    ) -> Result<(), OglError> {
        let mut f = OpenOptions::new()
            .read(true)
            .create(false)
            .open(file_name)
            .map_err(|e| {
                Report::new(OglError::InvalidData)
                    .attach_printable(format!("Failed to open {file_name}:{e}"))
            })?;

        let mut buf: Vec<u8> = Vec::new();
        f.read_to_end(&mut buf).map_err(|e| {
            Report::new(OglError::InvalidData)
                .attach_printable(format!("Fails to read {file_name}: {e}"))
        })?;

        self.create_from_buffer(&buf, gl, filter)
    }

    pub unsafe fn create_from_buffer(
        &mut self,
        buffer: &[u8],
        gl: &gl33::GlFns,
        filter: Texture2DFilter,
    ) -> Result<(), OglError> {
        // stb_image loads the image with the origin at top left. while the origin is at bottom
        // left in Texture. We need to flip the image here.
        stbi_set_flip_vertically_on_load(1);

        let mut width = 0;
        let mut height = 0;
        let mut bpp = 0;

        let data = stbi_load_from_memory(
            buffer.as_ptr(),
            buffer.len() as i32,
            &mut width,
            &mut height,
            &mut bpp,
            0,
        );
        if data.is_null() {
            return Err(Report::new(OglError::InvalidData)
                .attach_printable(format!("Failed to load image data")));
        }

        let data = slice::from_raw_parts(data, (width * height * bpp) as usize).to_owned();

        let mut id = 0;
        gl.GenTextures(1, &mut id);
        gl.BindTexture(gl33::GL_TEXTURE_2D, id);

        let format = match bpp {
            3 => gl33::GL_RGB,
            4 => gl33::GL_RGBA,
            _ => {
                return Err(Report::new(OglError::InvalidData)
                    .attach_printable(format!("Invalid bpp {bpp}")))
            }
        };

        gl.TexImage2D(
            gl33::GL_TEXTURE_2D,
            0,
            format.0 as i32,
            width,
            height,
            0,
            format,
            gl33::GL_UNSIGNED_BYTE,
            data.as_ptr().cast(),
        );

        let use_filter = match filter {
            Texture2DFilter::Linear => gl33::GL_LINEAR,
            Texture2DFilter::Nearest => gl33::GL_NEAREST,
            Texture2DFilter::NearestMiMapNearest => {
                gl.GenerateMipmap(gl33::GL_TEXTURE_2D);
                gl33::GL_NEAREST_MIPMAP_NEAREST
            }
            Texture2DFilter::NearestMiMapLinear => {
                gl.GenerateMipmap(gl33::GL_TEXTURE_2D);
                gl33::GL_NEAREST_MIPMAP_LINEAR
            }
            Texture2DFilter::LinearMiMapNearest => {
                gl.GenerateMipmap(gl33::GL_TEXTURE_2D);
                gl33::GL_LINEAR_MIPMAP_NEAREST
            }
            Texture2DFilter::LinearMiMapLinear => {
                gl.GenerateMipmap(gl33::GL_TEXTURE_2D);
                gl33::GL_LINEAR_MIPMAP_LINEAR
            }
        };

        gl.TexParameteri(
            gl33::GL_TEXTURE_2D,
            gl33::GL_TEXTURE_MIN_FILTER,
            use_filter.0 as i32,
        );
        gl.TexParameteri(
            gl33::GL_TEXTURE_2D,
            gl33::GL_TEXTURE_MAG_FILTER,
            gl33::GL_LINEAR.0 as i32,
        );
        gl.TexParameteri(
            gl33::GL_TEXTURE_2D,
            gl33::GL_TEXTURE_WRAP_S,
            gl33::GL_CLAMP_TO_EDGE.0 as i32,
        );
        gl.TexParameteri(
            gl33::GL_TEXTURE_2D,
            gl33::GL_TEXTURE_WRAP_T,
            gl33::GL_CLAMP_TO_EDGE.0 as i32,
        );

        gl.BindBuffer(gl33::GL_TEXTURE_2D, 0);

        self.id = id;
        self.data = data;
        self.width = width;
        self.height = height;
        self.bpp = bpp;

        Ok(())
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn bpp(&self) -> i32 {
        self.bpp
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub unsafe fn bind(&self, gl: &gl33::GlFns, slot: i32, location: i32) -> Result<(), OglError> {
        let s = match slot {
            0 => gl33::GL_TEXTURE0,
            1 => gl33::GL_TEXTURE1,
            2 => gl33::GL_TEXTURE2,
            3 => gl33::GL_TEXTURE3,
            4 => gl33::GL_TEXTURE4,
            5 => gl33::GL_TEXTURE5,
            6 => gl33::GL_TEXTURE6,
            7 => gl33::GL_TEXTURE7,
            _ => {
                return Err(Report::new(OglError::InvalidData)
                    .attach_printable(format!("slot {} is not supported", slot)))
            }
        };

        gl.ActiveTexture(s);
        gl.BindTexture(gl33::GL_TEXTURE_2D, self.id);
        gl.Uniform1i(location, slot);

        Ok(())
    }

    pub unsafe fn unbind(&self, gl: &gl33::GlFns) -> Result<(), OglError> {
        gl.BindTexture(gl33::GL_TEXTURE_2D, 0);
        Ok(())
    }
}

impl Display for Texture2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = format!(
            "width:{}, height:{}, bpp: {}, id: {}",
            self.width, self.height, self.bpp, self.id
        );

        write!(f, "{msg}")
    }
}

#[derive(Default, Debug, Clone)]
pub struct CubeFace {
    pub data: Vec<u8>,
    pub width: i32,
    pub height: i32,
    pub bpp: i32,
}

#[derive(Default, Debug, Clone)]
pub struct Texture2DCubeMap {
    id: u32,
    faces: [CubeFace; 6],
}

impl Texture2DCubeMap {
    pub unsafe fn create_from_file(
        &mut self,
        files: &[&str],
        gl: &gl33::GlFns,
        filter: Texture2DFilter,
    ) -> Result<(), OglError> {
        if files.len() < 6 {
            return Err(Report::new(OglError::InvalidData));
        }

        let mut buffers = vec![];
        for i in 0..6 {
            let mut f = OpenOptions::new()
                .read(true)
                .create(false)
                .open(files[i])
                .map_err(|e| {
                    Report::new(OglError::InvalidData)
                        .attach_printable(format!("Failed to open {}:{e}", files[i]))
                })?;

            let mut buf: Vec<u8> = Vec::new();
            f.read_to_end(&mut buf).map_err(|e| {
                Report::new(OglError::InvalidData)
                    .attach_printable(format!("Fails to read {}: {e}", files[i]))
            })?;

            buffers.push(buf);
        }

        self.create_from_buffer(buffers.iter().map(|a| a.as_slice()).collect(), gl, filter)
    }

    pub unsafe fn create_from_buffer(
        &mut self,
        buffers: Vec<&[u8]>,
        gl: &gl33::GlFns,
        filter: Texture2DFilter,
    ) -> Result<(), OglError> {
        if buffers.len() < 6 {
            return Err(Report::new(OglError::InvalidData));
        }

        gl.GenTextures(1, &mut self.id);
        gl.BindTexture(gl33::GL_TEXTURE_CUBE_MAP, self.id);

        for i in 0..6 {
            let mut width = 0;
            let mut height = 0;
            let mut bpp = 0;

            let data = stbi_load_from_memory(
                buffers[i].as_ptr(),
                buffers[i].len() as i32,
                &mut width,
                &mut height,
                &mut bpp,
                0,
            );

            if data.is_null() {
                return Err(Report::new(OglError::InvalidData)
                    .attach_printable("Failed to load image data"));
            }

            let format = match bpp {
                3 => gl33::GL_RGB,
                4 => gl33::GL_RGBA,
                _ => {
                    return Err(Report::new(OglError::InvalidData)
                        .attach_printable(format!("Invalid bpp {bpp}")))
                }
            };

            // 0 => gl33::GL_TEXTURE_CUBE_MAP_POSITIVE_X,
            // 1 => gl33::GL_TEXTURE_CUBE_MAP_NEGATIVE_X,
            // 2 => gl33::GL_TEXTURE_CUBE_MAP_POSITIVE_Y,
            // 3 => gl33::GL_TEXTURE_CUBE_MAP_NEGATIVE_Y,
            // 4 => gl33::GL_TEXTURE_CUBE_MAP_POSITIVE_Z,
            // 5 => gl33::GL_TEXTURE_CUBE_MAP_NEGATIVE_Z,
            gl.TexImage2D(
                gl33::GLenum(gl33::GL_TEXTURE_CUBE_MAP_NEGATIVE_X.0 + i as u32),
                0,
                format.0 as i32,
                width,
                height,
                0,
                format,
                gl33::GL_UNSIGNED_BYTE,
                data.cast(),
            );

            self.faces[i].data =
                std::slice::from_raw_parts(data, (width * height * bpp) as usize).to_owned();
            self.faces[i].width = width;
            self.faces[i].height = height;
            self.faces[i].bpp = bpp;
        }

        let use_filter = match filter {
            Texture2DFilter::Linear => gl33::GL_LINEAR,
            Texture2DFilter::Nearest => gl33::GL_NEAREST,
            Texture2DFilter::NearestMiMapNearest => {
                gl.GenerateMipmap(gl33::GL_TEXTURE_2D);
                gl33::GL_NEAREST_MIPMAP_NEAREST
            }
            Texture2DFilter::NearestMiMapLinear => {
                gl.GenerateMipmap(gl33::GL_TEXTURE_2D);
                gl33::GL_NEAREST_MIPMAP_LINEAR
            }
            Texture2DFilter::LinearMiMapNearest => {
                gl.GenerateMipmap(gl33::GL_TEXTURE_2D);
                gl33::GL_LINEAR_MIPMAP_NEAREST
            }
            Texture2DFilter::LinearMiMapLinear => {
                gl.GenerateMipmap(gl33::GL_TEXTURE_2D);
                gl33::GL_LINEAR_MIPMAP_LINEAR
            }
        };

        gl.TexParameteri(
            gl33::GL_TEXTURE_CUBE_MAP,
            gl33::GL_TEXTURE_MIN_FILTER,
            use_filter.0 as i32,
        );

        gl.TexParameteri(
            gl33::GL_TEXTURE_CUBE_MAP,
            gl33::GL_TEXTURE_MAG_FILTER,
            gl33::GL_LINEAR.0 as i32,
        );

        gl.TexParameteri(
            gl33::GL_TEXTURE_CUBE_MAP,
            gl33::GL_TEXTURE_WRAP_S,
            gl33::GL_CLAMP_TO_EDGE.0 as i32,
        );
        gl.TexParameteri(
            gl33::GL_TEXTURE_CUBE_MAP,
            gl33::GL_TEXTURE_WRAP_T,
            gl33::GL_CLAMP_TO_EDGE.0 as i32,
        );

        gl.TexParameteri(
            gl33::GL_TEXTURE_CUBE_MAP,
            gl33::GL_TEXTURE_WRAP_R,
            gl33::GL_CLAMP_TO_EDGE.0 as i32,
        );

        //gl.BindTexture(gl33::GL_TEXTURE_CUBE_MAP, 0);

        Ok(())
    }

    pub fn right(&self) -> &CubeFace {
        &self.faces[0]
    }

    pub fn left(&self) -> &CubeFace {
        &self.faces[1]
    }

    pub fn top(&self) -> &CubeFace {
        &self.faces[2]
    }

    pub fn bottom(&self) -> &CubeFace {
        &self.faces[3]
    }

    pub fn back(&self) -> &CubeFace {
        &self.faces[4]
    }

    pub fn front(&self) -> &CubeFace {
        &self.faces[5]
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub unsafe fn bind(&self, gl: &gl33::GlFns, slot: i32, location: i32) -> Result<(), OglError> {
        let s = match slot {
            0 => gl33::GL_TEXTURE0,
            1 => gl33::GL_TEXTURE1,
            2 => gl33::GL_TEXTURE2,
            3 => gl33::GL_TEXTURE3,
            4 => gl33::GL_TEXTURE4,
            5 => gl33::GL_TEXTURE5,
            6 => gl33::GL_TEXTURE6,
            7 => gl33::GL_TEXTURE7,
            _ => {
                return Err(Report::new(OglError::InvalidData)
                    .attach_printable(format!("slot {} is not supported", slot)))
            }
        };

        gl.ActiveTexture(s);
        gl.BindTexture(gl33::GL_TEXTURE_CUBE_MAP, self.id);
        gl.Uniform1i(location, slot);

        Ok(())
    }

    pub unsafe fn unbind(&self, _gl: &gl33::GlFns) -> Result<(), OglError> {
        //gl.BindTexture(gl33::GL_TEXTURE_CUBE_MAP, 0);
        Ok(())
    }
}

impl Display for Texture2DCubeMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut msg = String::new();
        msg.push_str(&format!("id: {}\n", self.id));

        (0..6_usize).into_iter().for_each(|i| {
            msg.push_str(&format!(
                "width:{}, height:{}, bpp: {}\n",
                self.faces[i].width, self.faces[i].height, self.faces[i].bpp
            ));
        });

        write!(f, "{msg}")
    }
}
