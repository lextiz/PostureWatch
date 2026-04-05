use crate::config::Config;

#[derive(Clone, Copy, Debug)]
pub enum FrameNotificationKind {
    BadPosture,
    DeskRaise,
    SessionLimit,
    DailyLimit,
    ApiSetup,
}

#[cfg(windows)]
pub fn show_frame_notification(kind: FrameNotificationKind, config: &Config) {
    use std::thread;

    if !config.frame_notification_enabled {
        return;
    }

    let color = frame_color_for_kind(kind, config);
    let duration_ms = config.frame_notification_duration_ms.clamp(100, 10_000);
    let area_ratio = (config.frame_notification_area_percent / 100.0).clamp(0.001, 0.25);

    thread::spawn(move || unsafe {
        show_frame_notification_blocking(color, duration_ms, area_ratio);
    });
}

#[cfg(not(windows))]
pub fn show_frame_notification(_kind: FrameNotificationKind, _config: &Config) {}

#[cfg(windows)]
fn frame_color_for_kind(kind: FrameNotificationKind, config: &Config) -> u32 {
    let raw = match kind {
        FrameNotificationKind::BadPosture => &config.frame_notification_bad_posture_color,
        FrameNotificationKind::DeskRaise => &config.frame_notification_desk_raise_color,
        FrameNotificationKind::SessionLimit => &config.frame_notification_session_limit_color,
        FrameNotificationKind::DailyLimit => &config.frame_notification_daily_limit_color,
        FrameNotificationKind::ApiSetup => &config.frame_notification_api_setup_color,
    };

    parse_hex_color(raw).unwrap_or(match kind {
        FrameNotificationKind::BadPosture => 0x0000FF,
        FrameNotificationKind::DeskRaise => 0x00A5FF,
        FrameNotificationKind::SessionLimit => 0x00CCFF,
        FrameNotificationKind::DailyLimit => 0xA020F0,
        FrameNotificationKind::ApiSetup => 0x0080FF,
    })
}

#[cfg(windows)]
fn parse_hex_color(raw: &str) -> Option<u32> {
    let hex = raw.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let rgb = u32::from_str_radix(hex, 16).ok()?;
    let r = (rgb >> 16) & 0xFF;
    let g = (rgb >> 8) & 0xFF;
    let b = rgb & 0xFF;

    Some(r | (g << 8) | (b << 16))
}

#[cfg(windows)]
unsafe fn show_frame_notification_blocking(color_bgr: u32, duration_ms: u64, area_ratio: f32) {
    use std::ffi::c_void;
    use std::sync::OnceLock;
    use std::time::{Duration, Instant};
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
    use windows_sys::Win32::Graphics::Gdi::{
        BeginPaint, CreateSolidBrush, DeleteObject, EndPaint, FillRect, PAINTSTRUCT,
    };
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetSystemMetrics,
        PeekMessageW, RegisterClassW, SetWindowLongPtrW, ShowWindow, TranslateMessage,
        UpdateWindow, CS_HREDRAW, CS_VREDRAW, GWLP_USERDATA, MSG, PM_REMOVE, SM_CXSCREEN,
        SM_CYSCREEN, SW_SHOWNOACTIVATE, WM_NCCREATE, WM_NCDESTROY, WM_PAINT, WNDCLASSW,
        WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
    };

    unsafe extern "system" fn frame_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        use std::ffi::c_void;
        use windows_sys::Win32::Foundation::RECT;
        use windows_sys::Win32::Graphics::Gdi::{
            BeginPaint, CreateSolidBrush, DeleteObject, EndPaint, FillRect, PAINTSTRUCT,
        };
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            DefWindowProcW, GetClientRect, GetWindowLongPtrW, GWLP_USERDATA, WM_NCCREATE,
            WM_NCDESTROY, WM_PAINT,
        };

        match msg {
            WM_NCCREATE => {
                let createstruct =
                    lparam as *const windows_sys::Win32::UI::WindowsAndMessaging::CREATESTRUCTW;
                if createstruct.is_null() {
                    return 0;
                }
                let color_ptr = unsafe { (*createstruct).lpCreateParams as *mut u32 };
                unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, color_ptr as isize) };
                1
            }
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = unsafe { std::mem::zeroed() };
                let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
                let color_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut u32 };
                let color = if color_ptr.is_null() {
                    0
                } else {
                    unsafe { *color_ptr }
                };
                let brush = unsafe { CreateSolidBrush(color) };
                let mut rect: RECT = RECT::default();
                unsafe { GetClientRect(hwnd, &mut rect) };
                unsafe { FillRect(hdc, &rect, brush) };
                unsafe { DeleteObject(brush as *mut c_void) };
                unsafe { EndPaint(hwnd, &ps) };
                0
            }
            WM_NCDESTROY => {
                let color_ptr = unsafe {
                    windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                        hwnd,
                        GWLP_USERDATA,
                    ) as *mut u32
                };
                if !color_ptr.is_null() {
                    let _ = unsafe { Box::from_raw(color_ptr) };
                    unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) };
                }
                unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
            }
            _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
        }
    }

    fn to_wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    static CLASS_NAME: &str = "PostureWatchFrameNotification";
    static CLASS_REGISTERED: OnceLock<bool> = OnceLock::new();

    let hinstance = GetModuleHandleW(std::ptr::null());
    if hinstance.is_null() {
        return;
    }

    let class_name_w = to_wide(CLASS_NAME);
    let _ = CLASS_REGISTERED.get_or_init(|| {
        let class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(frame_proc),
            hInstance: hinstance,
            lpszClassName: class_name_w.as_ptr(),
            ..Default::default()
        };
        RegisterClassW(&class) != 0
    });

    let screen_w = GetSystemMetrics(SM_CXSCREEN);
    let screen_h = GetSystemMetrics(SM_CYSCREEN);
    if screen_w <= 0 || screen_h <= 0 {
        return;
    }

    let thickness = border_thickness(screen_w, screen_h, area_ratio);
    let thickness = thickness.clamp(1, (screen_w.min(screen_h) / 2).max(1));

    let rects = [
        (0, 0, screen_w, thickness),
        (0, screen_h - thickness, screen_w, thickness),
        (0, thickness, thickness, screen_h - 2 * thickness),
        (
            screen_w - thickness,
            thickness,
            thickness,
            screen_h - 2 * thickness,
        ),
    ];

    let mut windows = Vec::with_capacity(4);
    for (x, y, w, h) in rects {
        if w <= 0 || h <= 0 {
            continue;
        }

        let color_ptr = Box::into_raw(Box::new(color_bgr));
        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class_name_w.as_ptr(),
            class_name_w.as_ptr(),
            WS_POPUP,
            x,
            y,
            w,
            h,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            hinstance,
            color_ptr as *mut c_void,
        );

        if hwnd.is_null() {
            let _ = Box::from_raw(color_ptr);
            continue;
        }

        ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        UpdateWindow(hwnd);
        windows.push(hwnd);
    }

    if windows.is_empty() {
        return;
    }

    let timeout = Instant::now() + Duration::from_millis(duration_ms);
    let mut msg: MSG = std::mem::zeroed();
    while Instant::now() < timeout {
        while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    for hwnd in windows {
        DestroyWindow(hwnd);
    }
}

#[cfg(windows)]
fn border_thickness(screen_w: i32, screen_h: i32, area_ratio: f32) -> i32 {
    let w = screen_w as f64;
    let h = screen_h as f64;
    let ratio = area_ratio as f64;

    let p = w + h;
    let discriminant = (p * p - 4.0 * ratio * w * h).max(0.0);
    let t = (p - discriminant.sqrt()) / 4.0;
    t.round() as i32
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    use super::parse_hex_color;

    #[cfg(windows)]
    #[test]
    fn parses_hex_colors_to_windows_bgr() {
        assert_eq!(parse_hex_color("#FF0000"), Some(0x0000FF));
        assert_eq!(parse_hex_color("00FF00"), Some(0x00FF00));
        assert_eq!(parse_hex_color("#0000FF"), Some(0xFF0000));
    }

    #[cfg(windows)]
    #[test]
    fn invalid_hex_colors_are_rejected() {
        assert_eq!(parse_hex_color("#GG0000"), None);
        assert_eq!(parse_hex_color("12345"), None);
        assert_eq!(parse_hex_color(""), None);
    }
}
