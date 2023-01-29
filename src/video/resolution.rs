use super::error::VideoError;
use error_stack::{IntoReport, Result};
use std::{fmt::Display, num::TryFromIntError};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Resolution {
    pub width: usize,
    pub height: usize,
}

impl Resolution {
    pub fn new(w: u32, h: u32) -> Result<Resolution, TryFromIntError> {
        let w: usize = w.try_into().into_report()?;
        let h: usize = h.try_into().into_report()?;
        Ok(Resolution {
            width: w,
            height: h,
        })
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
