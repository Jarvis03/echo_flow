#[cfg(not(target_os = "windows"))]
pub type TargetWindow = ();

#[cfg(target_os = "windows")]
pub type TargetWindow = isize;

#[cfg(not(target_os = "windows"))]
pub fn foreground_target() -> TargetWindow {}

#[cfg(not(target_os = "windows"))]
pub fn type_text(_target: TargetWindow, _text: &str) -> bool {
    false
}

#[cfg(target_os = "windows")]
pub fn foreground_target() -> TargetWindow {
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    unsafe { GetForegroundWindow().0 as isize }
}

#[cfg(target_os = "windows")]
pub fn type_text(target: TargetWindow, text: &str) -> bool {
    use std::{ffi::c_void, thread, time::Duration};
    use windows::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{GetForegroundWindow, SetForegroundWindow},
    };

    if target == 0 || text.is_empty() || text.chars().any(|character| character as u32 > 0xffff) {
        return false;
    }

    let target = HWND(target as *mut c_void);
    if unsafe { GetForegroundWindow() } != target {
        if !unsafe { SetForegroundWindow(target) }.as_bool() {
            return false;
        }
        thread::sleep(Duration::from_millis(80));
    }

    if unsafe { GetForegroundWindow() } != target {
        return false;
    }

    let expected = text.chars().count().saturating_mul(2) as u32;
    winput::send_str(text) == expected
}
