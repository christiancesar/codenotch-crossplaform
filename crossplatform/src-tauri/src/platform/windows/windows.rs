use ::windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
use ::windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FlashWindowEx, GetForegroundWindow, GetWindowRect, GetWindowTextLengthW,
    GetWindowThreadProcessId, IsIconic, IsWindowVisible, SetForegroundWindow, ShowWindow, FLASHWINFO,
    FLASHW_ALL, SW_RESTORE,
};

/// Visible top-level windows with a title: `(hwnd, owner pid, area)`.
pub fn visible_titled() -> Vec<(isize, u32, i64)> {
    unsafe extern "system" fn cb(hwnd: HWND, l: LPARAM) -> BOOL {
        let v = &mut *(l.0 as *mut Vec<(isize, u32, i64)>);
        if IsWindowVisible(hwnd).as_bool() && GetWindowTextLengthW(hwnd) > 0 {
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            let mut r = RECT::default();
            let area = if GetWindowRect(hwnd, &mut r).is_ok() {
                (r.right - r.left) as i64 * (r.bottom - r.top) as i64
            } else {
                0
            };
            v.push((hwnd.0 as isize, pid, area));
        }
        BOOL(1)
    }
    let mut out: Vec<(isize, u32, i64)> = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(cb), LPARAM(&mut out as *mut _ as isize));
    }
    out
}

/// Restore, bring to front and flash twice so the eye finds it.
pub fn raise(h: isize) -> bool {
    unsafe {
        let hwnd = HWND(h as *mut core::ffi::c_void);
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        let _ = SetForegroundWindow(hwnd);
        let fi = FLASHWINFO {
            cbSize: std::mem::size_of::<FLASHWINFO>() as u32,
            hwnd,
            dwFlags: FLASHW_ALL,
            uCount: 2,
            dwTimeout: 0,
        };
        let _ = FlashWindowEx(&fi);
    }
    true
}

pub fn foreground_pid() -> u32 {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return 0;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        pid
    }
}
