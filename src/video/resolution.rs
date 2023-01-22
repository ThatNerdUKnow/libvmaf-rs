use std::fmt::Display;

#[derive(Debug)]
pub struct Resolution {
    pub width: usize,
    pub height: usize,
}

impl GetResolution for Resolution {
    fn get_resolution(&self) -> &Resolution {
        self
    }
}

impl Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Width: {}, Height: {}", self.width, self.height)
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
