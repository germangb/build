#[cfg(feature = "d2")]
use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use std::ops::{Deref, DerefMut};

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 200;

/// Frame render content.
pub type Frame = [[u32; WIDTH]; HEIGHT];

// TODO(german): find a better way to interop' with the eg crate
pub(crate) struct EGFrame<'a>(pub &'a mut Frame);

#[cfg(feature = "d2")]
impl DrawTarget<Rgb888> for EGFrame<'_> {
    type Error = std::convert::Infallible;

    fn draw_pixel(&mut self, Pixel(point, color): Pixel<Rgb888>) -> Result<(), Self::Error> {
        if point.x >= 0 && point.x < (WIDTH as i32) && point.y >= 0 && point.y < (HEIGHT as i32) {
            self.0[point.y as usize][point.x as usize] =
                (color.r() as u32) << 16 | (color.g() as u32) << 8 | (color.b() as u32);
        }
        Ok(())
    }

    fn size(&self) -> Size {
        Size::new(WIDTH as _, HEIGHT as _)
    }
}
