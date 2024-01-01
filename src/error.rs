use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum OglError {
    WaylandError,
    SDLError,
    EglError,
    GlError,
    InvalidData,
    Unexpected,
}

impl Display for OglError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            OglError::WaylandError => "Wayland error",
            OglError::SDLError => "SDL error",
            OglError::EglError => "EGL error",
            OglError::GlError => "Opengl error",
            OglError::InvalidData => "Invalid error",
            OglError::Unexpected => "Unexpected error",
        };

        write!(f, "{msg}")
    }
}

impl Error for OglError {}
