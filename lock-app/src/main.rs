// Standard Lock - background interceptor.
// "Nothing opens without permission."
//
// Launch modes:
//   1. `standard-lock.exe "C:\path\to\file.txt"`
//        This is how Windows launches us: it's the registered handler
//        for locked extensions. We show the password prompt, and on
//        success hand the file to its real, original application.
//   2. `standard-lock.exe` (no args)
//        Runs quietly in the system tray. Exists so the user can see
//        at a glance that protection is active, jump to the Home app,
//        or quit. Not required for interception itself to work - each
//        locked file double-click spawns its own short-lived instance
//        of this exe per mode (1) above.

#![windows_subsystem = "windows"] // no console window

mod dialog;
mod tray;

use std::path::PathBuf;
use standard_lock_core::Config;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() >= 2 {
        handle_file_open(PathBuf::from(&args[1]));
    } else {
        tray::run();
    }
}

fn handle_file_open(file: PathBuf) {
    let cfg = match Config::load() {
        Ok(c) => c,
        Err(_) => return, // can't read config, fail closed (deny)
    };

    if !cfg.is_locked(&file) {
        // Not actually flagged for locking (e.g. in the exceptions list).
        // Let it straight through to its original app.
        let ext = file
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default();
        let _ = standard_lock_core::open_with_original_handler(ext, &file, &cfg);
        return;
    }

    if cfg.password_hash.is_empty() {
        dialog::show_message(
            "Standard Lock",
            "No password has been set yet. Open \"Standard Lock - Home\" to set one up before locking files.",
        );
        return; // deny by default
    }

    let ext = file
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_string();

    let file_name = file
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "this file".to_string());

    if dialog::prompt_password(&file_name, &cfg.password_hash) {
        let _ = standard_lock_core::open_with_original_handler(&ext, &file, &cfg);
    }
    // else: user cancelled or failed too many attempts -> do nothing (deny).
}
