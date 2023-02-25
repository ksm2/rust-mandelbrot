#![feature(test)]

mod color;
mod complex;
mod dim;

use crate::color::Color;
use crate::complex::Complex;
use crate::dim::Dim;
use rayon::prelude::*;
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreatedHDC, EndPaint,
    SelectObject, SetPixel, PAINTSTRUCT, SRCCOPY,
};
use windows::{
    core::*, Win32::Foundation::*, Win32::System::LibraryLoader::GetModuleHandleA,
    Win32::UI::WindowsAndMessaging::*,
};

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleA(None)?;
        debug_assert!(instance.0 != 0);

        let window_class = s!("window");

        let wc = WNDCLASSA {
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hInstance: instance,
            lpszClassName: window_class,

            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        let atom = RegisterClassA(&wc);
        debug_assert!(atom != 0);

        CreateWindowExA(
            WINDOW_EX_STYLE::default(),
            window_class,
            s!("Mandelbrot"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            900,
            600,
            None,
            None,
            instance,
            None,
        );

        let mut message = MSG::default();

        while GetMessageA(&mut message, HWND(0), 0, 0).into() {
            DispatchMessageA(&message);
        }

        Ok(())
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_ERASEBKGND => LRESULT(1),
            WM_PAINT => {
                println!("WM_PAINT");

                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(window, &mut ps);

                let mut rect = RECT::default();
                GetClientRect(window, &mut rect);
                let width = rect.right;
                let height = rect.bottom;

                let bm_dc = CreateCompatibleDC(hdc);
                let back_buffer = CreateCompatibleBitmap(hdc, width, height);
                SelectObject(bm_dc, back_buffer);

                paint(bm_dc, Dim::new(width, height));

                BitBlt(hdc, 0, 0, width, height, bm_dc, 0, 0, SRCCOPY);

                EndPaint(window, &ps);

                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}

unsafe fn paint(ctx: CreatedHDC, dim: Dim) {
    let pixels = calculate_pixels_par(dim);

    for (x, y, color) in pixels {
        SetPixel(ctx, x, y, COLORREF(color.into()));
    }
}

#[allow(dead_code)]
fn calculate_pixels(dim: Dim) -> Vec<(i32, i32, Color)> {
    dim.into_iter()
        .map(|(x, y)| calculate_pixel(&dim, x, y))
        .collect::<Vec<_>>()
}

fn calculate_pixels_par(dim: Dim) -> Vec<(i32, i32, Color)> {
    dim.into_par_iter()
        .map(|(x, y)| calculate_pixel(&dim, x, y))
        .collect::<Vec<_>>()
}

fn calculate_pixel(dim: &Dim, x: i32, y: i32) -> (i32, i32, Color) {
    let c = translate(dim.width, dim.height, x, y);
    let mut z = c;

    let max_iter = 400;
    let mut iter = 0;
    let mut inside = true;
    loop {
        let new_z = z * z + c;
        if new_z.abs() >= 2.0 {
            inside = false;
            break;
        }

        z = new_z;
        if iter > max_iter {
            break;
        }
        iter += 1;
    }

    let color = if inside {
        Color::BLACK
    } else {
        color_iter(iter)
    };

    (x, y, color)
}

fn translate(width: i32, height: i32, x: i32, y: i32) -> Complex {
    let third_width = width / 3;
    let half_height = height / 2;

    let value_x = (x - third_width - third_width) as f64 / width as f64 * 3.0;
    let value_y = (y - half_height) as f64 / height as f64 * 2.0;

    (Complex::new(value_x, value_y) / 6.0) + Complex::new(-0.75, -0.25)
}

fn color_iter(iter: i32) -> Color {
    if iter <= 100 {
        let it_rel = (iter as f64) / 100.0;
        Color::blend(Color::BLUE, Color::WHITE, it_rel)
    } else if iter <= 200 {
        let it_rel = (iter - 100) as f64 / 100.0;
        Color::blend(Color::WHITE, Color::RED, it_rel)
    } else if iter <= 300 {
        let it_rel = (iter - 200) as f64 / 100.0;
        Color::blend(Color::RED, Color::YELLOW, it_rel)
    } else {
        let it_rel = (iter - 300) as f64 / 100.0;
        Color::blend(Color::YELLOW, Color::CYAN, it_rel)
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use crate::dim::Dim;
    use crate::{calculate_pixels, calculate_pixels_par};
    use test::Bencher;

    #[bench]
    fn bench_pixel_calc(b: &mut Bencher) {
        b.iter(|| calculate_pixels(Dim::new(60, 40)))
    }

    #[bench]
    fn bench_pixel_calc_par(b: &mut Bencher) {
        b.iter(|| calculate_pixels_par(Dim::new(60, 40)))
    }
}
