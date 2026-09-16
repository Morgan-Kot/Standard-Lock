//! Helper definitions for win32 UI control references.

use windows::Win32::Foundation::HWND;

pub struct ControlHandles {
    pub main_window: HWND,
    pub status_label: HWND,
}

impl ControlHandles {
    pub fn new(main_window: HWND, status_label: HWND) -> Self {
        Self {
            main_window,
            status_label,
        }
    }
}

pub struct Ctx {
    pub handles: ControlHandles,
    pub title: String,
}

pub fn create_context(main_window: HWND, status_label: HWND, title: &str) -> Box<Ctx> {
    Box::new(Ctx {
        handles: ControlHandles::new(main_window, status_label),
        title: title.to_string(),
    })
}