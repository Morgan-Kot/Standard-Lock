#![windows_subsystem = "windows"]

use std::path::{Path, PathBuf};
use windows::Win32::UI::Shell::{SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST};

mod ui;
use ui::run_window;

pub const ID_SAVE_PW_BTN: i32 = 201;
pub const ID_CUR_PW: i32 = 202;
pub const ID_NEW_PW: i32 = 203;
pub const ID_CONFIRM_PW: i32 = 204;

pub const ID_LOCKALL_CHECK: i32 = 210;
pub const ID_EXT_LIST: i32 = 211;
pub const ID_EXT_EDIT: i32 = 212;
pub const ID_EXT_ADD_BTN: i32 = 213;
pub const ID_EXT_REMOVE_BTN: i32 = 214;

pub const ID_FILES_LIST: i32 = 220;
pub const ID_FILES_ADD_BTN: i32 = 221;
pub const ID_FILES_REMOVE_BTN: i32 = 222;

pub const ID_LOCK_EXE_BTN: i32 = 230;
pub const ID_APPLY_BTN: i32 = 240;

pub struct AppState {
    pub cfg: standard_lock_core::Config,
    pub lock_exe: Option<PathBuf>,
}

impl Default for AppState {
    fn default() -> Self {
        let cfg = standard_lock_core::Config::load().unwrap_or_default();
        let lock_exe = standard_lock_core::app_registry::get_lock_exe_path();
        AppState { cfg, lock_exe }
    }
}

pub fn on_save_password(
    state: &mut AppState,
    current: &str,
    new: &str,
    confirm: &str,
) -> String {
    if !state.cfg.password_hash.is_empty()
        && !standard_lock_core::verify_password(current, &state.cfg.password_hash)
    {
        return "Current password incorrect.".to_string();
    }
    if new.is_empty() {
        return "New password cannot be empty.".to_string();
    }
    if new != confirm {
        return "New password and confirmation do not match.".to_string();
    }
    match standard_lock_core::hash_password(new) {
        Ok(h) => {
            state.cfg.password_hash = h;
            let _ = state.cfg.save();
            "Password set successfully.".to_string()
        }
        Err(e) => format!("Error hashing password: {e}"),
    }
}

pub fn on_pick_lock_exe(state: &mut AppState) -> Option<PathBuf> {
    use rfd::FileDialog;
    if let Some(path) = FileDialog::new()
        .add_filter("Executable", &["exe"])
        .set_title("Locate standard-lock.exe")
        .pick_file()
    {
        state.lock_exe = Some(path.clone());
        let _ = standard_lock_core::app_registry::set_lock_exe_path(&path);
        return Some(path);
    }
    state.lock_exe.clone()
}

pub fn on_add_extension(state: &mut AppState, raw: &str) -> String {
    let clean = raw.trim().trim_start_matches('.').to_lowercase();
    if clean.is_empty() {
        return "Extension cannot be empty.".to_string();
    }
    if state.cfg.locked_extensions.contains(&clean) {
        return format!(".{clean} is already in the list.");
    }
    state.cfg.locked_extensions.push(clean.clone());
    format!("Added .{clean}")
}

pub fn on_remove_extension(state: &mut AppState, ext: &str) {
    let clean = ext.trim_start_matches('.').to_lowercase();
    state.cfg.locked_extensions.retain(|e| e != &clean);
}

pub fn on_add_files(state: &mut AppState) -> usize {
    use rfd::FileDialog;
    if let Some(paths) = FileDialog::new()
        .set_title("Select Files to Add")
        .pick_files()
    {
        let count = paths.len();
        for path in paths {
            let p_str = path.to_string_lossy().to_string();
            if state.cfg.lock_all {
                if !state.cfg.exceptions.contains(&p_str) {
                    state.cfg.exceptions.push(p_str);
                }
            } else {
                if !state.cfg.explicit_locked_paths.contains(&p_str) {
                    state.cfg.explicit_locked_paths.push(p_str);
                }
            }
        }
        return count;
    }
    0
}

pub fn on_remove_file(state: &mut AppState, path: &str) {
    let target = path.to_lowercase();
    state
        .cfg
        .exceptions
        .retain(|p| p.to_lowercase() != target);
    state
        .cfg
        .explicit_locked_paths
        .retain(|p| p.to_lowercase() != target);
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
        "Exceptions (unlocked files):"
    } else {
        "Explicitly Locked Files:"
    }
}

pub fn on_apply(state: &mut AppState) -> String {
    if state.cfg.password_hash.is_empty() {
        return "Set a password first before applying.".to_string();
    }

    let lock_exe = match &state.lock_exe {
        Some(p) => p.clone(),
        None => return "Please locate standard-lock.exe first.".to_string(),
    };

    if let Err(e) = state.cfg.save() {
        return format!("Failed to save config: {e}");
    }

    let exts = state.cfg.locked_extensions.clone();
    for ext in exts {
        let _ = standard_lock_core::registry::lock_extension(
            &ext,
            &lock_exe,
            &mut state.cfg,
        );
    }

    unsafe {
        SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None);
    }

    "Settings applied and Windows Shell refreshed successfully.".to_string()
}

fn main() {
    if let Ok(exe) = std::env::current_exe() {
        let _ = standard_lock_core::app_registry::set_home_path(&exe);
    }

    let mut state = AppState::default();
    unsafe {
        run_window(&mut state);
    }
}
