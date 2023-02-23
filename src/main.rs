use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreatedHDC, EndPaint, FillRect,
    SelectObject, SetPixel, COLOR_WINDOW, HBRUSH, PAINTSTRUCT, SRCCOPY,
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
            CW_USEDEFAULT,
            CW_USEDEFAULT,
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

                paint(bm_dc, &ps.rcPaint, width, height);

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

unsafe fn paint(ctx: CreatedHDC, paint_rect: &RECT, width: i32, height: i32) {
    FillRect(ctx, paint_rect, HBRUSH(COLOR_WINDOW.0 as isize));
    for x in paint_rect.left..=paint_rect.right {
        let value_x = x as f64 / width as f64;
        for y in paint_rect.top..=paint_rect.bottom {
            let value_y = (y as f64) / (height) as f64;
            let color = blend(value_y, 0x0000FF00, blend(value_x, 0x000000FF, 0x00FF0000));
            SetPixel(ctx, x, y, COLORREF(color));
        }
    }
}

fn blend(value: f64, from: u32, to: u32) -> u32 {
    let red1 = from & 0xFF;
    let red2 = to & 0xFF;
    let red = blend_comp(value, red1, red2);

    let green1 = (from >> 8) & 0xFF;
    let green2 = (to >> 8) & 0xFF;
    let green = blend_comp(value, green1, green2);

    let blue1 = (from >> 16) & 0xFF;
    let blue2 = (to >> 16) & 0xFF;
    let blue = blend_comp(value, blue1, blue2);

    let output = red + (green << 8) + (blue << 16);
    output
}

fn blend_comp(value: f64, c1: u32, c2: u32) -> u32 {
    (c1 as f64 * (1.0 - value) + (c2 as f64 * value)).round() as u32
}
