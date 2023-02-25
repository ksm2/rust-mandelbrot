use crate::color::Color;
use crate::dim::Dim;

pub trait Bitmap {
    fn dim(&self) -> Dim;
    fn set_pixel(&mut self, x: i32, y: i32, color: Color);
}
