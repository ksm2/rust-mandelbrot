use crate::color::Color;
use crate::dim::Dim;

pub trait Bitmap {
    fn dim(&self) -> Dim;
    fn set_pixel(&mut self, x: i32, y: i32, color: Color);
}

pub struct InMemBitmap<const W: usize, const H: usize> {
    pixels: [[Color; W]; H],
}

impl<const W: usize, const H: usize> InMemBitmap<W, H> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let pixels = [[Color::BLACK; W]; H];
        Self { pixels }
    }
}

impl<const W: usize, const H: usize> Bitmap for InMemBitmap<W, H> {
    fn dim(&self) -> Dim {
        Dim::new(W as i32, H as i32)
    }

    fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        self.pixels[y as usize][x as usize] = color
    }
}
