use std::path::PathBuf;
use windows::core::PCWSTR;
use windows::Win32::UI::Controls::Dialogs::*;

const BUF_LEN: usize = 32768; // generous; supports many multi-selected files

fn to_wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Opens a single-file picker. `filter_ext`, if given (e.g. "exe"),
/// restricts the file list to that extension.
pub fn pick_single_file(filter_ext: Option<&str>) -> Option<PathBuf> {
    let mut buf = vec![0u16; BUF_LEN];
    let filter = match filter_ext {
        Some(ext) => to_wide(&format!(
            "{ext} files\0*.{ext}\0All files\0*.*\0\0"
        )),
        None => to_wide("All files\0*.*\0\0"),
    };

    let mut ofn = OPENFILENAMEW {
        lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
        lpstrFilter: PCWSTR(filter.as_ptr()),
        lpstrFile: windows::core::PWSTR(buf.as_mut_ptr()),
        nMaxFile: BUF_LEN as u32,
        Flags: OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_EXPLORER,
        ..Default::default()
    };

    let ok = unsafe { GetOpenFileNameW(&mut ofn) };
    if ok.as_bool() {
        let len = buf.iter().position(|&c| c == 0).unwrap_or(0);
        Some(PathBuf::from(String::from_utf16_lossy(&buf[..len])))
    } else {
        None
    }
}

/// Opens a multi-select file picker and returns full absolute paths.
pub fn pick_multiple_files() -> Vec<PathBuf> {
    let mut buf = vec![0u16; BUF_LEN];
    let filter = to_wide("All files\0*.*\0\0");

    let mut ofn = OPENFILENAMEW {
        lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
        lpstrFilter: PCWSTR(filter.as_ptr()),
        lpstrFile: windows::core::PWSTR(buf.as_mut_ptr()),
        nMaxFile: BUF_LEN as u32,
        Flags: OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_EXPLORER | OFN_ALLOWMULTISELECT,
        ..Default::default()
    };

    let ok = unsafe { GetOpenFileNameW(&mut ofn) };
    if !ok.as_bool() {
        return Vec::new();
    }

    // Explorer-style multi-select result: directory, NUL, file1, NUL,
    // file2, NUL, ..., NUL NUL. If only one file was picked, the
    // buffer is just that one full path with a single NUL terminator.
    let mut parts: Vec<String> = Vec::new();
    let mut start = 0usize;
    for i in 0..buf.len() {
        if buf[i] == 0 {
            if i == start {
                break; // double-NUL: end of list
            }
            parts.push(String::from_utf16_lossy(&buf[start..i]));
            start = i + 1;
        }
    }

    if parts.len() <= 1 {
        return parts.into_iter().map(PathBuf::from).collect();
    }

    let dir = PathBuf::from(&parts[0]);
    parts[1..].iter().map(|f| dir.join(f)).collect()
}
