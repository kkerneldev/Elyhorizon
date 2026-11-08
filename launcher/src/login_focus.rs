//! Restore only the existing window, once a validated Ely login was saved successfully.
use super::*;

impl Launcher {
    pub(super) fn return_after_ely_login(&self, cx: &mut Context<Self>) {
        let handle = self.aside.window_for_login();
        cx.defer(move |cx| {
            #[cfg(not(target_os = "windows"))]
            cx.activate(true);
            let _ = handle.update(cx, |_, window, _| restore(window));
        });
    }
}

#[cfg(target_os = "windows")]
fn restore(window: &Window) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        FlashWindowEx, IsIconic, SetForegroundWindow, ShowWindowAsync, FLASHWINFO,
        FLASHW_TIMERNOFG, FLASHW_TRAY, SW_RESTORE,
    };
    let Ok(handle) = HasWindowHandle::window_handle(window) else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };
    let hwnd = handle.hwnd.get() as *mut core::ffi::c_void;
    // Called on the UI thread with its live window handle. Do not synthesize input or
    // attach to the browser thread to circumvent Windows foreground restrictions.
    unsafe {
        if IsIconic(hwnd) != 0 {
            ShowWindowAsync(hwnd, SW_RESTORE);
        }
        if SetForegroundWindow(hwnd) == 0 {
            let info = FLASHWINFO {
                cbSize: std::mem::size_of::<FLASHWINFO>() as u32,
                hwnd,
                dwFlags: FLASHW_TRAY | FLASHW_TIMERNOFG,
                uCount: 3,
                dwTimeout: 0,
            };
            FlashWindowEx(&info);
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn restore(window: &Window) {
    // GPUI delegates to AppKit on macOS and the active X11/Wayland compositor on Linux.
    // Compositors may refuse activation without a recent user activation token. Attention
    // is requested once so the completed login remains discoverable in that case.
    window.request_attention();
    window.activate_window();
}
