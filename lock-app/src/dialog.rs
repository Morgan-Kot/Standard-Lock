//! Small, dependency-free (beyond `windows`) native password dialog.

use std::cell::RefCell;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{SetFocus, VK_ESCAPE, VK_RETURN};
use windows::Win32::UI::WindowsAndMessaging::*;

const ID_EDIT: i32 = 101;
const ID_OK: i32 = 102;
const ID_CANCEL: i32 = 103;

const MAX_ATTEMPTS: u32 = 5;

struct DialogState {
    hwnd_edit: HWND,
    hwnd_status: HWND,
    hash: String,
    attempts: u32,
    success: bool,
}

thread_local! {
    static STATE: RefCell<Option<DialogState>> = RefCell::new(None);
}

pub fn prompt_password(file_name: &str, hash: &str) -> bool {
    unsafe {
        let hmodule = windows::Win32::System::LibraryLoader::GetModuleHandleW(None).unwrap();
        let hinstance: HINSTANCE = hmodule.into();
        let class_name = w!("StandardLockPasswordPrompt");

        let wc = WNDCLASSW {
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance,
            lpszClassName: class_name,
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH(
                (windows::Win32::Graphics::Gdi::COLOR_WINDOW.0 + 1) as *mut _,
            ),
            ..Default::default()
        };
        RegisterClassW(&wc);

        let (sw, sh) = (
            GetSystemMetrics(SM_CXSCREEN),
            GetSystemMetrics(SM_CYSCREEN),
        );
        let (w_, h_) = (340, 190);
        let (x, y) = ((sw - w_) / 2, (sh - h_) / 2);

        let title_wide = to_wide("Standard Lock");
        let hwnd = CreateWindowExW(
            WS_EX_DLGMODALFRAME | WS_EX_TOPMOST,
            class_name,
            PCWSTR(title_wide.as_ptr()),
            WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
            x,
            y,
            w_,
            h_,
            None,
            None,
            hinstance,
            None,
        )
        .expect("failed to create dialog window");

        make_static(
            hwnd,
            hinstance,
            &format!("\"{file_name}\" is locked."),
            16,
            16,
            300,
            20,
            true,
        );
        make_static(
            hwnd,
            hinstance,
            standard_lock_core::MOTTO,
            16,
            38,
            300,
            16,
            false,
        );
        make_static(hwnd, hinstance, "Password:", 16, 66, 300, 16, false);

        let hwnd_edit = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("EDIT"),
            PCWSTR::null(),
            WS_CHILD | WS_VISIBLE | WS_BORDER | WINDOW_STYLE(ES_PASSWORD as u32),
            16,
            84,
            300,
            24,
            hwnd,
            HMENU(ID_EDIT as *mut _),
            hinstance,
            None,
        )
        .expect("failed to create edit control");

        let hwnd_status = make_static(hwnd, hinstance, "", 16, 112, 300, 18, false);

        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("BUTTON"),
            w!("Unlock"),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_DEFPUSHBUTTON as u32),
            140,
            140,
            85,
            28,
            hwnd,
            HMENU(ID_OK as *mut _),
            hinstance,
            None,
        )
        .ok();

        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("BUTTON"),
            w!("Cancel"),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_PUSHBUTTON as u32),
            231,
            140,
            85,
            28,
            hwnd,
            HMENU(ID_CANCEL as *mut _),
            hinstance,
            None,
        )
        .ok();

        STATE.with(|s| {
            *s.borrow_mut() = Some(DialogState {
                hwnd_edit,
                hwnd_status,
                hash: hash.to_string(),
                attempts: 0,
                success: false,
            });
        });

        let _ = SetFocus(hwnd_edit);
        let _ = ShowWindow(hwnd, SW_SHOW);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if msg.message == WM_KEYDOWN {
                if msg.wParam.0 == VK_RETURN.0 as usize {
                    submit(hwnd);
                    continue;
                } else if msg.wParam.0 == VK_ESCAPE.0 as usize {
                    let _ = DestroyWindow(hwnd);
                    continue;
                }
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        STATE.with(|s| s.borrow().as_ref().map(|d| d.success).unwrap_or(false))
    }
}

unsafe fn make_static(
    parent: HWND,
    hinstance: HINSTANCE,
    text: &str,
    x: i32,
    y: i32,
    w_: i32,
    h_: i32,
    bold: bool,
) -> HWND {
    let wide = to_wide(text);
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("STATIC"),
        PCWSTR(wide.as_ptr()),
        WS_CHILD | WS_VISIBLE,
        x,
        y,
        w_,
        h_,
        parent,
        None,
        hinstance,
        None,
    )
    .unwrap_or_default();
    let _ = bold;
    hwnd
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let id = (wparam.0 & 0xffff) as i32;
            if id == ID_OK {
                submit(hwnd);
            } else if id == ID_CANCEL {
                let _ = DestroyWindow(hwnd);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn submit(hwnd: HWND) {
    STATE.with(|s| {
        let mut borrow = s.borrow_mut();
        let state = match borrow.as_mut() {
            Some(s) => s,
            None => return,
        };

        let entered = unsafe { get_window_text(state.hwnd_edit) };
        let ok = standard_lock_core::verify_password(&entered, &state.hash);

        if ok {
            state.success = true;
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
            return;
        }

        state.attempts += 1;
        let remaining = MAX_ATTEMPTS.saturating_sub(state.attempts);
        unsafe {
            set_window_text(state.hwnd_edit, "");
            let _ = SetFocus(state.hwnd_edit);
            if remaining == 0 {
                set_window_text(state.hwnd_status, "Too many attempts.");
                let _ = DestroyWindow(hwnd);
            } else {
                set_window_text(
                    state.hwnd_status,
                    &format!("Incorrect password. {remaining} attempt(s) left."),
                );
            }
        }
    });
}

pub fn show_message(title: &str, body: &str) {
    unsafe {
        let _ = MessageBoxW(
            None,
            PCWSTR(to_wide(body).as_ptr()),
            PCWSTR(to_wide(title).as_ptr()),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}

fn to_wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

unsafe fn get_window_text(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    let len = GetWindowTextW(hwnd, &mut buf);
    String::from_utf16_lossy(&buf[..len.max(0) as usize])
}

unsafe fn set_window_text(hwnd: HWND, text: &str) {
    let wide = to_wide(text);
    let _ = SetWindowTextW(hwnd, PCWSTR(wide.as_ptr()));
}