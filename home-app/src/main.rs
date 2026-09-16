// Standard Lock - Home
// "Nothing opens without permission."
//
// Lets the user:
//   * set / change the password
//   * choose which file TYPES are intercepted (locked_extensions)
//   * choose "lock everything of those types" vs "only lock files I
//     pick individually"
//   * add/remove individual files from the exceptions list (when
//     locking everything) or the explicit lock list (when not)
//   * apply changes, which updates config.json AND the Windows
//     registry associations for the affected extensions.

#![windows_subsystem = "windows"]

mod controls;
mod filedialog;

use std::path::PathBuf;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::*;

use standard_lock_core::{app_registry, hash_password, registry, verify_password, Config};

// --- Control IDs ---------------------------------------------------
const ID_CUR_PW: i32 = 201;
const ID_NEW_PW: i32 = 202;
const ID_CONFIRM_PW: i32 = 203;
const ID_SAVE_PW_BTN: i32 = 204;
const ID_PW_STATUS: i32 = 205;

const ID_LOCKALL_CHECK: i32 = 210;

const ID_EXT_LIST: i32 = 220;
const ID_EXT_EDIT: i32 = 221;
const ID_EXT_ADD_BTN: i32 = 222;
const ID_EXT_REMOVE_BTN: i32 = 223;

const ID_FILES_LABEL: i32 = 230;
const ID_FILES_LIST: i32 = 231;
const ID_FILES_ADD_BTN: i32 = 232;
const ID_FILES_REMOVE_BTN: i32 = 233;

const ID_APPLY_BTN: i32 = 240;
const ID_APPLY_STATUS: i32 = 241;

const ID_LOCK_EXE_BTN: i32 = 250;

struct AppState {
    cfg: Config,
    lock_exe: Option<PathBuf>,
}

fn main() {
    // Let the tray icon find us later.
    if let Ok(self_path) = std::env::current_exe() {
        let _ = app_registry::set_home_path(&self_path);
    }

    let cfg = Config::load().unwrap_or_default();
    let lock_exe = app_registry::get_lock_exe_path();

    let mut state = AppState { cfg, lock_exe };

    unsafe { controls::run_window(&mut state) };
}

// ---------------------------------------------------------------------
// All the "what happens when a button is clicked" logic lives here,
// separate from the raw window/control plumbing in controls.rs.
// ---------------------------------------------------------------------

pub fn on_save_password(state: &mut AppState, current: &str, new: &str, confirm: &str) -> String {
    if !state.cfg.password_hash.is_empty() && !verify_password(current, &state.cfg.password_hash)
    {
        return "Current password is incorrect.".into();
    }
    if new.is_empty() {
        return "New password can't be empty.".into();
    }
    if new != confirm {
        return "New password and confirmation don't match.".into();
    }
    match hash_password(new) {
        Ok(h) => {
            state.cfg.password_hash = h;
            match state.cfg.save() {
                Ok(_) => "Password saved.".into(),
                Err(e) => format!("Saved in memory but failed to write config.json: {e}"),
            }
        }
        Err(e) => format!("Failed to hash password: {e}"),
    }
}

pub fn on_pick_lock_exe(state: &mut AppState) -> Option<PathBuf> {
    let path = filedialog::pick_single_file(Some("exe"))?;
    let _ = app_registry::set_lock_exe_path(&path);
    state.lock_exe = Some(path.clone());
    Some(path)
}

pub fn on_add_extension(state: &mut AppState, raw: &str) -> String {
    let ext = raw.trim().trim_start_matches('.').to_lowercase();
    if ext.is_empty() {
        return "Enter a file extension first (e.g. csv).".into();
    }
    if ext.eq_ignore_ascii_case("exe") {
        return "Executables can't be intercepted (by design - Windows won't allow it safely)."
            .into();
    }
    if state.cfg.locked_extensions.iter().any(|e| *e == ext) {
        return format!(".{ext} is already in the list.");
    }
    state.cfg.locked_extensions.push(ext.clone());
    format!(".{ext} added. Click Apply Changes to activate it.")
}

pub fn on_remove_extension(state: &mut AppState, ext: &str) {
    let ext = ext.trim_start_matches('.').to_lowercase();
    state.cfg.locked_extensions.retain(|e| *e != ext);
}

pub fn on_add_files(state: &mut AppState) -> usize {
    let files = filedialog::pick_multiple_files();
    let target = if state.cfg.lock_all {
        &mut state.cfg.exceptions
    } else {
        &mut state.cfg.explicit_locked_paths
    };
    let mut added = 0;
    for f in files {
        let s = f.to_string_lossy().to_string();
        if !target.iter().any(|p| p.eq_ignore_ascii_case(&s)) {
            target.push(s);
            added += 1;
        }
    }
    added
}

pub fn on_remove_file(state: &mut AppState, path: &str) {
    if state.cfg.lock_all {
        state.cfg.exceptions.retain(|p| p != path);
    } else {
        state.cfg.explicit_locked_paths.retain(|p| p != path);
    }
}

pub fn current_file_list<'a>(state: &'a AppState) -> &'a [String] {
    if state.cfg.lock_all {
        &state.cfg.exceptions
    } else {
        &state.cfg.explicit_locked_paths
    }
}

pub fn files_list_caption(state: &AppState) -> &'static str {
    if state.cfg.lock_all {
        "Exceptions (never ask for a password, even though the type above is locked):"
    } else {
        "Files that require a password (only these, even though the type above is locked):"
    }
}

/// Writes config.json and brings the Windows registry in line with
/// `cfg.locked_extensions`: registers new ones, unregisters ones that
/// were removed since the last Apply.
pub fn on_apply(state: &mut AppState) -> String {
    if state.cfg.password_hash.is_empty() {
        return "Set a password before locking any files.".into();
    }
    let lock_exe = match &state.lock_exe {
        Some(p) => p.clone(),
        None => return "Locate standard-lock.exe first (the Lock app's exe file).".into(),
    };
    if !lock_exe.exists() {
        return "That standard-lock.exe path no longer exists. Please re-select it.".into();
    }

    // Anything currently backed up in original_handlers that is no
    // longer in locked_extensions must be unregistered/restored.
    let currently_registered: Vec<String> = state.cfg.original_handlers.keys().cloned().collect();
    for ext in currently_registered {
        if !state.cfg.locked_extensions.iter().any(|e| *e == ext) {
            if let Err(e) = registry::unlock_extension(&ext, &mut state.cfg) {
                return format!("Failed to unlock .{ext}: {e}");
            }
        }
    }

    // Anything in locked_extensions that isn't registered yet, register.
    let to_register: Vec<String> = state
        .cfg
        .locked_extensions
        .iter()
        .filter(|e| !state.cfg.original_handlers.contains_key(*e))
        .cloned()
        .collect();
    for ext in to_register {
        if let Err(e) = registry::lock_extension(&ext, &lock_exe, &mut state.cfg) {
            return format!("Failed to lock .{ext}: {e}");
        }
    }

    match state.cfg.save() {
        Ok(_) => "Applied. File associations updated.".into(),
        Err(e) => format!("Applied to Windows but failed to save config.json: {e}"),
    }
}
