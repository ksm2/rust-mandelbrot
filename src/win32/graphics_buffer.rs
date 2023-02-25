use std::mem::size_of;
use std::ptr::null_mut;
use windows::Win32::Graphics::Gdi::{
    CreateDIBSection, DeleteObject, GetDC, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    HBITMAP,
};

pub struct GraphicsBuffer {
    pub hbm: HBITMAP,
    pub data: *mut u32,
}

impl GraphicsBuffer {
    pub fn new(wd: i32, hgt: i32) -> Self {
        unsafe {
            let hdc_screen = GetDC(None);
            let mut bmi = BITMAPINFO::default();
            bmi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
            bmi.bmiHeader.biWidth = wd;
            bmi.bmiHeader.biHeight = -hgt; // top-down
            bmi.bmiHeader.biPlanes = 1;
            bmi.bmiHeader.biBitCount = 32;
            bmi.bmiHeader.biCompression = BI_RGB;

            let mut data = null_mut();
            let hbm =
                CreateDIBSection(hdc_screen, &bmi, DIB_RGB_COLORS, &mut data, None, 0).unwrap();

            let data = data as *mut u32;

            Self { hbm, data }
        }
    }
}

impl Drop for GraphicsBuffer {
    fn drop(&mut self) {
        unsafe {
            DeleteObject(self.hbm);
        }
    }
}
