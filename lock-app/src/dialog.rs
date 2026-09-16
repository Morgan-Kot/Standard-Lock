use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{SetFocus, VK_ESCAPE, VK_RETURN};
use windows::Win32::UI::WindowsAndMessaging::*;

const ID_EDIT_PW: i32 = 101;
const ID_BTN_OK: i32 = 102;
const ID_BTN_CANCEL: i32 = 103;

struct DialogState {
    entered_password: Option<String>,
    hwnd_edit: HWND,
}

pub fn prompt_password() -> Option<String> {
    unsafe {
        let hmodule = GetModuleHandleW(None).ok()?;
        let class_name = w!("StandardLockDialog");

        let wc = WNDCLASSW {
            lpfnWndProc: Some(dialog_wnd_proc),
            hInstance: hmodule.into(),
            lpszClassName: class_name,
            hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH(COLOR_WINDOW.0 as _),
            ..Default::default()
        };
        RegisterClassW(&wc);

        let mut state = DialogState {
            entered_password: None,
            hwnd_edit: HWND::default(),
        };

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name,
            w!("Standard Lock - Authorization"),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            360,
            160,
            None,
            None,
            hmodule,
            Some(&mut state as *mut _ as _),
        );

        if hwnd.0 == 0 {
            return None;
        }

        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = UpdateWindow(hwnd);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if msg.message == WM_KEYDOWN {
                if msg.wParam.0 == VK_RETURN.0 as usize {
                    let _ = SendMessageW(hwnd, WM_COMMAND, WPARAM(ID_BTN_OK as usize), LPARAM(0));
                    continue;
                } else if msg.wParam.0 == VK_ESCAPE.0 as usize {
                    let _ = SendMessageW(hwnd, WM_COMMAND, WPARAM(ID_BTN_CANCEL as usize), LPARAM(0));
                    continue;
                }
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        state.entered_password
    }
}

unsafe extern "system" fn dialog_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let create_struct = &*(lparam.0 as *const CREATESTRUCTW);
            let state_ptr = create_struct.lpCreateParams as *mut DialogState;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);

            let hInstance = create_struct.hInstance;

            let _ = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("STATIC"),
                w!("Enter password to unlock file:"),
                WS_CHILD | WS_VISIBLE,
                15,
                15,
                310,
                20,
                hwnd,
                None,
                hInstance,
                None,
            );

            let hwnd_edit = CreateWindowExW(
                WS_EX_CLIENTEDGE,
                w!("EDIT"),
                w!(""),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(ES_PASSWORD as u32) | WINDOW_STYLE(ES_AUTOHSCROLL as u32),
                15,
                40,
                315,
                25,
                hwnd,
                HMENU(ID_EDIT_PW as isize),
                hInstance,
                None,
            );

            let _ = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!("OK"),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_DEFPUSHBUTTON as u32),
                170,
                80,
,
                25,
                hwnd,
                HMENU(ID_BTN_OK as isize),
                hInstance,
                None,
            );

            let _ = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!("Cancel"),
                WS_CHILD | WS_VISIBLE,
                255,
                80,
                75,
                25,
                hwnd,
                HMENU(ID_BTN_CANCEL as isize),
                hInstance,
                None,
            );

            if !state_ptr.is_null() {
                (*state_ptr).hwnd_edit = hwnd_edit;
            }
            let _ = SetFocus(hwnd_edit);
            LRESULT(0)
        }
        WM_COMMAND => {
            let id = (wparam.0 & 0xffff) as i32;
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut DialogState;

            if !ptr.is_null() {
                let state = &mut *ptr;
                if id == ID_BTN_OK {
                    let len = GetWindowTextLengthW(state.hwnd_edit);
                    let mut buf = vec![0u16; (len + 1) as usize];
                    GetWindowTextW(state.hwnd_edit, &mut buf);
                    state.entered_password = Some(String::from_utf16_lossy(&buf[..len as usize]));
                    let _ = DestroyWindow(hwnd);
                } else if id == ID_BTN_CANCEL {
                    state.entered_password = None;
                    let _ = DestroyWindow(hwnd);
                }
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
