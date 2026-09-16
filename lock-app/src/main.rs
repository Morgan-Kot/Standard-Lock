#![windows_subsystem = "windows"]

mod dialog;
mod tray;

use std::env;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();

    // If executed with arguments, treat the first arg as a file path to prompt password for
    if args.len() > 1 {
        let target_file = &args[1];
        let cfg = standard_lock_core::Config::load().unwrap_or_default();

        if cfg.password_hash.is_empty() {
            let _ = Command::new("cmd")
                .args(["/C", "start", "", target_file])
                .spawn();
            return;
        }

        if let Some(password) = dialog::prompt_password() {
            if standard_lock_core::verify_password(&password, &cfg.password_hash) {
                let _ = Command::new("cmd")
                    .args(["/C", "start", "", target_file])
                    .spawn();
            } else {
                unsafe {
                    use windows::core::w;
                    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
                    MessageBoxW(
                        None,
                        w!("Incorrect password. Access denied."),
                        w!("Standard Lock"),
                        MB_OK | MB_ICONERROR,
                    );
                }
            }
        }
        return;
    }

    // Otherwise, start tray icon process
    tray::run();
}
