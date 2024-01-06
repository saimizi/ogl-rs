pub mod error;
pub mod texture2d;

pub trait VertexOps {
    fn to_u8_slice(&self) -> &[u8];
}

impl VertexOps for [f32] {
    fn to_u8_slice(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self.as_ptr() as *const u8,
                self.len() * core::mem::size_of::<f32>(),
            )
        }
    }
}

impl VertexOps for [u16] {
    fn to_u8_slice(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self.as_ptr() as *const u8,
                self.len() * core::mem::size_of::<u16>(),
            )
        }
    }
}

impl VertexOps for Vec<f32> {
    fn to_u8_slice(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self.as_ptr() as *const u8,
                self.len() * core::mem::size_of::<f32>(),
            )
        }
    }
}
