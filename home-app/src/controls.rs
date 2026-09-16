use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::*;

struct Handles {
    hwnd: HWND,
    hinstance: HINSTANCE,
    cur_pw: HWND,
    new_pw: HWND,
    confirm_pw: HWND,
    pw_status: HWND,
    lockall_check: HWND,
    ext_list: HWND,
    ext_edit: HWND,
    files_label: HWND,
    files_list: HWND,
    apply_status: HWND,
    lock_exe_status: HWND,
}

struct Ctx {
    state: AppState,
    h: Handles,
}

pub unsafe fn run_window(state: &mut AppState) {
    // We move ownership of `state` into Ctx for the duration of the
    // window's life, then hand back control when it closes. Simpler
    // than threading a borrow through every Win32 callback.
    let state_owned = std::mem::replace(
        state,
        AppState {
            cfg: Config::load().unwrap_or_default(),
            lock_exe: None,
        },
    );

    let hmodule = windows::Win32::System::LibraryLoader::GetModuleHandleW(None).unwrap();
    let hinstance: HINSTANCE = hmodule.into();
    let class_name = w!("StandardLockHome");

    let wc = WNDCLASSW {
        lpfnWndProc: Some(wnd_proc),
        hInstance: hinstance,
        lpszClassName: class_name,
        hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
        hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH(
            (windows::Win32::Graphics::Gdi::COLOR_BTNFACE.0 + 1) as *mut _,
        ),
        ..Default::default()
    };
    RegisterClassW(&wc);

    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        class_name,
        w!("Standard Lock - Home"),
        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_VISIBLE,
        200,
        100,
        500,
        620,
        None,
        None,
        hinstance,
        None,
    )
    .expect("failed to create main window");

    let h = build_controls(hwnd, hinstance);

    let mut ctx = Box::new(Ctx {
        state: state_owned,
        h,
    });
    populate_from_state(&ctx.h, &ctx.state);
    SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(ctx) as isize);

    let mut msg = MSG::default();
    while GetMessageW(&mut msg, None, 0, 0).as_bool() {
        let _ = TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    // Retrieve state back out (best-effort; process is exiting anyway).
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Ctx;
    if !ptr.is_null() {
        let ctx = Box::from_raw(ptr);
        *state = ctx.state;
    }
}

unsafe fn build_controls(hwnd: HWND, hinstance: HINSTANCE) -> Handles {
    let label = |text: &str, x, y, w_, h_| {
        let wide = to_wide(text);
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("STATIC"),
            PCWSTR(wide.as_ptr()),
            WS_CHILD | WS_VISIBLE,
            x,
            y,
            w_,
            h_,
            hwnd,
            None,
            hinstance,
            None,
        )
        .unwrap_or_default()
    };
    let edit = |id: i32, x, y, w_, h_, password: bool| {
        let style = WS_CHILD | WS_VISIBLE | WS_BORDER
            | WINDOW_STYLE(if password { 0x20 /* ES_PASSWORD */ } else { 0 });
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("EDIT"),
            PCWSTR::null(),
            style,
            x,
            y,
            w_,
            h_,
            hwnd,
            HMENU(id as *mut _),
            hinstance,
            None,
        )
        .unwrap_or_default()
    };
    let button = |text: &str, id: i32, x, y, w_, h_| {
        let wide = to_wide(text);
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("BUTTON"),
            PCWSTR(wide.as_ptr()),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_PUSHBUTTON as u32),
            x,
            y,
            w_,
            h_,
            hwnd,
            HMENU(id as *mut _),
            hinstance,
            None,
        )
        .unwrap_or_default()
    };

    label("Standard Lock - Home  |  Nothing opens without permission.", 16, 10, 450, 18);

    label("Password", 16, 38, 200, 16);
    label("Current password (blank if none set yet):", 16, 58, 300, 16);
    let cur_pw = edit(ID_CUR_PW, 16, 76, 440, 24, true);
    label("New password:", 16, 106, 300, 16);
    let new_pw = edit(ID_NEW_PW, 16, 124, 440, 24, true);
    label("Confirm new password:", 16, 154, 300, 16);
    let confirm_pw = edit(ID_CONFIRM_PW, 16, 172, 440, 24, true);
    let _save_btn = button("Save Password", ID_SAVE_PW_BTN, 16, 202, 140, 26);
    let pw_status = label("", 166, 206, 290, 18);

    label("Locked file types", 16, 244, 300, 16);
    let lockall_check = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("BUTTON"),
        w!("Lock ALL files of these types (uncheck to pick individual files instead)"),
        WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
        16,
        262,
        440,
        20,
        hwnd,
        HMENU(ID_LOCKALL_CHECK as *mut _),
        hinstance,
        None,
    )
    .unwrap_or_default();

    let ext_list = CreateWindowExW(
        WINDOW_EX_STYLE(0) | WS_EX_CLIENTEDGE,
        w!("LISTBOX"),
        PCWSTR::null(),
        WS_CHILD | WS_VISIBLE | WS_VSCROLL | WINDOW_STYLE(LBS_NOTIFY as u32),
        16,
        288,
        200,
        90,
        hwnd,
        HMENU(ID_EXT_LIST as *mut _),
        hinstance,
        None,
    )
    .unwrap_or_default();
    let ext_edit = edit(ID_EXT_EDIT, 226, 288, 100, 24, false);
    let _add_ext = button("Add Type", ID_EXT_ADD_BTN, 226, 316, 100, 26);
    let _rm_ext = button("Remove Type", ID_EXT_REMOVE_BTN, 226, 346, 100, 26);

    let files_label = label(files_list_caption_default(), 16, 388, 440, 16);
    let files_list = CreateWindowExW(
        WINDOW_EX_STYLE(0) | WS_EX_CLIENTEDGE,
        w!("LISTBOX"),
        PCWSTR::null(),
        WS_CHILD | WS_VISIBLE | WS_VSCROLL | WINDOW_STYLE(LBS_NOTIFY as u32),
        16,
        406,
        440,
        110,
        hwnd,
        HMENU(ID_FILES_LIST as *mut _),
        hinstance,
        None,
    )
    .unwrap_or_default();
    let _add_file = button("Add File(s)...", ID_FILES_ADD_BTN, 16, 522, 130, 26);
    let _rm_file = button("Remove Selected", ID_FILES_REMOVE_BTN, 152, 522, 130, 26);

    let _lock_exe_btn = button("Locate standard-lock.exe...", ID_LOCK_EXE_BTN, 16, 558, 180, 26);
    let lock_exe_status = label("", 202, 562, 254, 18);

    let _apply_btn = button("Apply Changes", ID_APPLY_BTN, 330, 522, 126, 26);
    let apply_status = label("", 16, 580, 440, 18);

    Handles {
        hwnd,
        hinstance,
        cur_pw,
        new_pw,
        confirm_pw,
        pw_status,
        lockall_check,
        ext_list,
        ext_edit,
        files_label,
        files_list,
        apply_status,
        lock_exe_status,
    }
}

fn files_list_caption_default() -> &'static str {
    "Exceptions (never ask for a password, even though the type above is locked):"
}

unsafe fn populate_from_state(h: &Handles, state: &AppState) {
    SendMessageW(h.ext_list, LB_RESETCONTENT, WPARAM(0), LPARAM(0));
    for ext in &state.cfg.locked_extensions {
        lb_add(h.ext_list, &format!(".{ext}"));
    }

    SendMessageW(
        h.lockall_check,
        BM_SETCHECK,
        WPARAM(if state.cfg.lock_all { 1 } else { 0 }),
        LPARAM(0),
    );

    set_text(h.files_label, files_list_caption(state));
    SendMessageW(h.files_list, LB_RESETCONTENT, WPARAM(0), LPARAM(0));
    for f in current_file_list(state) {
        lb_add(h.files_list, f);
    }

    if let Some(p) = &state.lock_exe {
        set_text(h.lock_exe_status, &p.to_string_lossy());
    } else {
        set_text(h.lock_exe_status, "(not set)");
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Ctx;
    if ptr.is_null() {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    let ctx = &mut *ptr;

    match msg {
        WM_COMMAND => {
            let id = (wparam.0 & 0xffff) as i32;
            let notify = (wparam.0 >> 16) as u32;
            handle_command(ctx, id, notify);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn handle_command(ctx: &mut Ctx, id: i32, _notify: u32) {
    match id {
        ID_SAVE_PW_BTN => {
            let cur = get_text(ctx.h.cur_pw);
            let new = get_text(ctx.h.new_pw);
            let confirm = get_text(ctx.h.confirm_pw);
            let msg = on_save_password(&mut ctx.state, &cur, &new, &confirm);
            set_text(ctx.h.pw_status, &msg);
            set_text(ctx.h.new_pw, "");
            set_text(ctx.h.confirm_pw, "");
            set_text(ctx.h.cur_pw, "");
        }
        ID_LOCKALL_CHECK => {
            let checked = SendMessageW(ctx.h.lockall_check, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 != 0;
            ctx.state.cfg.lock_all = checked;
            set_text(ctx.h.files_label, files_list_caption(&ctx.state));
            refresh_files_list(ctx);
        }
        ID_EXT_ADD_BTN => {
            let raw = get_text(ctx.h.ext_edit);
            let msg = on_add_extension(&mut ctx.state, &raw);
            set_text(ctx.h.ext_edit, "");
            set_text(ctx.h.apply_status, &msg);
            refresh_ext_list(ctx);
        }
        ID_EXT_REMOVE_BTN => {
            if let Some(sel) = lb_get_selected_text(ctx.h.ext_list) {
                on_remove_extension(&mut ctx.state, &sel);
                refresh_ext_list(ctx);
            }
        }
        ID_FILES_ADD_BTN => {
            let n = on_add_files(&mut ctx.state);
            set_text(ctx.h.apply_status, &format!("{n} file(s) added."));
            refresh_files_list(ctx);
        }
        ID_FILES_REMOVE_BTN => {
            if let Some(sel) = lb_get_selected_text(ctx.h.files_list) {
                on_remove_file(&mut ctx.state, &sel);
                refresh_files_list(ctx);
            }
        }
        ID_LOCK_EXE_BTN => {
            if let Some(p) = on_pick_lock_exe(&mut ctx.state) {
                set_text(ctx.h.lock_exe_status, &p.to_string_lossy());
            }
        }
        ID_APPLY_BTN => {
            let msg = on_apply(&mut ctx.state);
            set_text(ctx.h.apply_status, &msg);
            refresh_ext_list(ctx);
        }
        _ => {}
    }
}

unsafe fn refresh_ext_list(ctx: &Ctx) {
    SendMessageW(ctx.h.ext_list, LB_RESETCONTENT, WPARAM(0), LPARAM(0));
    for ext in &ctx.state.cfg.locked_extensions {
        lb_add(ctx.h.ext_list, &format!(".{ext}"));
    }
}

unsafe fn refresh_files_list(ctx: &Ctx) {
    SendMessageW(ctx.h.files_list, LB_RESETCONTENT, WPARAM(0), LPARAM(0));
    for f in current_file_list(&ctx.state) {
        lb_add(ctx.h.files_list, f);
    }
}

// --- small Win32 helpers --------------------------------------------

unsafe fn lb_add(hwnd: HWND, text: &str) {
    let wide = to_wide(text);
    SendMessageW(hwnd, LB_ADDSTRING, WPARAM(0), LPARAM(wide.as_ptr() as isize));
}

unsafe fn lb_get_selected_text(hwnd: HWND) -> Option<String> {
    let idx = SendMessageW(hwnd, LB_GETCURSEL, WPARAM(0), LPARAM(0)).0;
    if idx < 0 {
        return None;
    }
    let len = SendMessageW(hwnd, LB_GETTEXTLEN, WPARAM(idx as usize), LPARAM(0)).0;
    if len <= 0 {
        return None;
    }
    let mut buf = vec![0u16; len as usize + 1];
    SendMessageW(
        hwnd,
        LB_GETTEXT,
        WPARAM(idx as usize),
        LPARAM(buf.as_mut_ptr() as isize),
    );
    let text = String::from_utf16_lossy(&buf[..len as usize]);
    // Extension list entries are shown as ".ext" - strip the dot back off
    // for callers that expect a bare extension; file paths are untouched
    // since they won't start with a lone dot.
    Some(text.trim_start_matches('.').to_string())
}

unsafe fn get_text(hwnd: HWND) -> String {
    let len = GetWindowTextLengthW(hwnd);
    if len <= 0 {
        return String::new();
    }
    let mut buf = vec![0u16; len as usize + 1];
    let actual = GetWindowTextW(hwnd, &mut buf);
    String::from_utf16_lossy(&buf[..actual.max(0) as usize])
}

unsafe fn set_text(hwnd: HWND, text: &str) {
    let wide = to_wide(text);
    let _ = SetWindowTextW(hwnd, PCWSTR(wide.as_ptr()));
}

fn to_wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
