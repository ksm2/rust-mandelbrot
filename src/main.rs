#![feature(test)]

mod bitmap;
mod color;
mod complex;
mod dim;
mod mandelbrot;
mod win32;

use crate::dim::Dim;
use crate::mandelbrot::calculate_pixels_par;
use crate::win32::Win32Bitmap;
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleDC, DeleteDC, EndPaint, SelectObject, PAINTSTRUCT, SRCCOPY,
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
            WM_SIZE => handle_size(window),
            WM_CREATE => handle_create(window),
            WM_PAINT => handle_paint(window),
            WM_DESTROY => handle_destroy(window),
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}

fn handle_size(window: HWND) -> LRESULT {
    handle_create(window)
}

fn handle_create(window: HWND) -> LRESULT {
    let dim = get_window_dimensions(window);
    let mut bitmap = Win32Bitmap::new(dim);

    // Draw in the background
    calculate_pixels_par(&mut bitmap);
    bitmap.swap();
    bitmap.clear();

    // Store bitmap on window handle
    set_bitmap(window, bitmap);

    LRESULT(0)
}

unsafe fn handle_paint(window: HWND) -> LRESULT {
    let bitmap = get_bitmap(window).as_mut().unwrap();

    let mut ps = PAINTSTRUCT::default();
    let hdc = BeginPaint(window, &mut ps);
    let bm_dc = CreateCompatibleDC(hdc);
    let old_bmp = SelectObject(bm_dc, bitmap.bitmap());

    let width = bitmap.dim.width;
    let height = bitmap.dim.height;
    BitBlt(hdc, 0, 0, width, height, bm_dc, 0, 0, SRCCOPY);

    SelectObject(hdc, old_bmp);
    DeleteDC(bm_dc);
    EndPaint(window, &ps);

    LRESULT(0)
}

unsafe fn handle_destroy(window: HWND) -> LRESULT {
    let bitmap = Box::from_raw(get_bitmap(window));
    drop(bitmap);
    PostQuitMessage(0);
    LRESULT(0)
}

fn get_window_dimensions(window: HWND) -> Dim {
    unsafe {
        let mut r = RECT::default();
        GetClientRect(window, &mut r);
        Dim::new(r.right - r.left, r.bottom - r.top)
    }
}

fn get_bitmap(window: HWND) -> *mut Win32Bitmap {
    unsafe {
        let bitmap = GetWindowLongPtrA(window, GWLP_USERDATA);
        bitmap as *mut Win32Bitmap
    }
}

fn set_bitmap(window: HWND, bitmap: Win32Bitmap) {
    unsafe {
        let user_data = Box::into_raw(Box::new(bitmap)) as isize;
        SetWindowLongPtrA(window, GWLP_USERDATA, user_data);
    }
}
