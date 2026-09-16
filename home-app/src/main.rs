use std::path::PathBuf;
use windows::Win32::UI::Shell::{SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST};

pub const ID_SET_PASSWORD: i32 = 201;
pub const ID_CURR_PW: i32 = 202;
pub const ID_NEW_PW: i32 = 203;
pub const ID_CONFIRM_PW: i32 = 204;

pub const ID_EXT_INPUT: i32 = 210;
pub const ID_ADD_EXT: i32 = 211;
pub const ID_EXT_LIST: i32 = 212;
pub const ID_REMOVE_EXT: i32 = 213;

pub const ID_RADIO_ALL: i32 = 220;
pub const ID_RADIO_LIST: i32 = 221;

pub const ID_FILES_LIST: i32 = 231;
pub const ID_ADD_FILES: i32 = 232;
pub const ID_REMOVE_FILE: i32 = 233;

pub const ID_APPLY: i32 = 240;

pub struct AppState {
    pub config: standard_lock_core::Config,
    pub current_password_input: String,
    pub new_password_input: String,
    pub confirm_password_input: String,
    pub lock_exe: PathBuf,
}

impl AppState {
    pub fn load() -> Self {
        let config = standard_lock_core::Config::load().unwrap_or_default();
        let lock_exe = standard_lock_core::app_registry::get_lock_exe_path()
            .unwrap_or_else(|| PathBuf::from("lock-app.exe"));
        AppState {
            config,
            current_password_input: String::new(),
            new_password_input: String::new(),
            confirm_password_input: String::new(),
            lock_exe,
        }
    }
}

pub fn on_save_password(
    state: &mut AppState,
    current: &str,
    new: &str,
    confirm: &str,
) -> String {
    if !state.config.password_hash.is_empty()
        && !standard_lock_core::verify_password(current, &state.config.password_hash)
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
            state.config.password_hash = h;
            let _ = state.config.save();
            "Password set successfully.".to_string()
        }
        Err(e) => format!("Error hashing password: {e}"),
    }
}

pub fn on_pick_lock_exe(state: &mut AppState) -> Option<PathBuf> {
    Some(state.lock_exe.clone())
}

pub fn on_add_extension(state: &mut AppState, raw: &str) -> String {
    let clean = raw.trim().trim_start_matches('.').to_lowercase();
    if clean.is_empty() {
        return "Extension cannot be empty.".to_string();
    }
    if state.config.locked_extensions.contains(&clean) {
        return format!(".{clean} is already in the list.");
    }
    state.config.locked_extensions.push(clean.clone());
    format!("Added .{clean}")
}

pub fn on_remove_extension(state: &mut AppState, ext: &str) {
    let clean = ext.trim_start_matches('.').to_lowercase();
    state.config.locked_extensions.retain(|e| e != &clean);
}

pub fn on_add_files(state: &mut AppState) -> usize {
    state.config.explicit_locked_paths.len()
}

pub fn on_remove_file(state: &mut AppState, path: &str) {
    let target = path.to_lowercase();
    state
        .config
        .exceptions
        .retain(|p| p.to_lowercase() != target);
    state
        .config
        .explicit_locked_paths
        .retain(|p| p.to_lowercase() != target);
}

pub fn current_file_list<'a>(state: &'a AppState) -> &'a [String] {
    if state.config.lock_all {
        &state.config.exceptions
    } else {
        &state.config.explicit_locked_paths
    }
}

pub fn files_list_caption(state: &AppState) -> &'static str {
    if state.config.lock_all {
        "Exceptions (unlocked files):"
    } else {
        "Explicitly Locked Files:"
    }
}

pub fn on_apply(state: &mut AppState) -> String {
    if state.config.password_hash.is_empty() {
        return "Set a password first before applying.".to_string();
    }

    if let Err(e) = state.config.save() {
        return format!("Failed to save config: {e}");
    }

    // Collect extensions to clone them and release the immutable borrow on state.config
    let extensions: Vec<String> = state.config.locked_extensions.clone();
    for ext in extensions {
        let _ = standard_lock_core::registry::lock_extension(
            &ext,
            &state.lock_exe,
            &mut state.config,
        );
    }

    unsafe {
        SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None);
    }

    "Settings applied and Windows Shell refreshed successfully.".to_string()
}

fn main() {
    println!("Standard Lock - Home");
}