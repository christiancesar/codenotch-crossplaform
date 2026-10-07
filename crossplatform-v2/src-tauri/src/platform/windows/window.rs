use ::windows::Win32::Foundation::HWND;
use ::windows::Win32::UI::WindowsAndMessaging::{GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW};

/// A console launch (`codenotch doctor`) prints to the terminal it came from; the GUI subsystem
/// otherwise has no console at all.
pub fn prepare_process() {
    use ::windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
    unsafe {
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

/// WS_EX_NOACTIVATE + WS_EX_TOOLWINDOW: never takes focus, never shows in Alt-Tab.
pub fn no_activate(w: &tauri::WebviewWindow) {
    let Ok(h) = w.hwnd() else { return };
    unsafe {
        let hwnd = HWND(h.0 as isize as *mut core::ffi::c_void);
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex | WS_EX_NOACTIVATE.0 as isize | WS_EX_TOOLWINDOW.0 as isize);
    }
}

pub fn left_button_down() -> bool {
    use ::windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
    unsafe { (GetAsyncKeyState(VK_LBUTTON.0 as i32) as u16 & 0x8000) != 0 }
}
