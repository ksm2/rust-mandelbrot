use crate::bitmap::Bitmap;
use crate::color::Color;
use crate::dim::Dim;
use graphics_buffer::GraphicsBuffer;
use windows::Win32::Graphics::Gdi::HBITMAP;

mod graphics_buffer;

pub struct Win32Bitmap {
    pub dim: Dim,
    pub front: GraphicsBuffer,
    pub back: GraphicsBuffer,
}

impl Win32Bitmap {
    pub fn new(dim: Dim) -> Self {
        let front = GraphicsBuffer::new(dim.width, dim.height);
        let back = GraphicsBuffer::new(dim.width, dim.height);
        Self { dim, front, back }
    }

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.front, &mut self.back)
    }

    pub fn bitmap(&self) -> HBITMAP {
        self.front.hbm
    }

    pub fn size(&self) -> usize {
        self.dim.len()
    }

    pub fn clear(&mut self) {
        unsafe {
            self.back.data.write_bytes(0, self.size());
        }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        unsafe {
            let offset = y * self.dim.width + x;
            self.back.data.offset(offset as isize).replace(color.into());
        }
    }
}

impl Bitmap for Win32Bitmap {
    fn dim(&self) -> Dim {
        self.dim
    }

    fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        self.set_pixel(x, y, color)
    }
}
