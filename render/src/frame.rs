use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use std::ops::Deref;

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 200;

#[derive(Debug)]
pub struct Frame(pub [[u32; WIDTH]; HEIGHT]);

impl Frame {
    /// Create a new empty frame.
    pub fn new() -> Box<Self> {
        Box::new(Frame([[0x000000; WIDTH]; HEIGHT]))
    }

    /// Clear frame contents.
    pub fn clear(&mut self) {
        self.0 = [[0; WIDTH]; HEIGHT];
    }
}

impl Deref for Frame {
    type Target = [[u32; 320]; 200];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DrawTarget<Rgb888> for Frame {
    type Error = std::convert::Infallible;

    fn draw_pixel(&mut self, Pixel(point, color): Pixel<Rgb888>) -> Result<(), Self::Error> {
        if point.x >= 0 && point.x < 320 && point.y >= 0 && point.y < 200 {
            self.0[point.y as usize][point.x as usize] =
                ((color.r() as u32) << 16 | (color.g() as u32) << 8 | (color.b() as u32));
        }
        Ok(())
    }

    fn size(&self) -> Size {
        Size::new(320, 200)
    }
}
