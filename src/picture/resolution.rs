use std::fmt::Display;

use self::error::ResolutionError;

pub mod error;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Resolution {
    pub width: usize,
    pub height: usize,
}

impl Resolution {
    pub fn new(w: u32, h: u32) -> Result<Resolution, ResolutionError> {
        let width: usize = w.try_into()?;
        let height: usize = h.try_into()?;

        // Guard against zero frame size
        if w == 0 || h == 0 {
            return Err(ResolutionError::InvalidResolution {
                width: w,
                height: h,
            });
        }

        Ok(Resolution { width, height })
    }
}

impl GetResolution for Resolution {
    fn get_resolution(&self) -> &Resolution {
        self
    }
}

impl Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

pub trait GetResolution {
    fn get_resolution(&self) -> &Resolution;
    fn get_width(&self) -> usize {
        self.get_resolution().width
    }
    fn get_height(&self) -> usize {
        self.get_resolution().height
    }
}
