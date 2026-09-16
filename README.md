# Standard Lock
"**Nothing opens without permission.**"

A two-part Windows utility, written in Rust:

| Folder      | Builds                    | Purpose                                                          |
|-------------|----------------------------|-------------------------------------------------------------------|
| `core`      | (library, not shipped)     | Shared config, password hashing, Windows file-association logic |
| `lock-app`  | `standard-lock.exe`        | **Standard Lock** — the background interceptor + tray icon       |
| `home-app`  | `standard-lock-home.exe`   | **Standard Lock - Home** — password + file selection console     |

Ship `lock-app`'s target folder as the "Standard Lock" install, and `home-app`'s
target folder as the "Standard Lock - Home" install — they're independent
binaries and can live anywhere, including two separate folders as you described.

## How interception works

Windows won't let a third-party process silently hook `.exe` launches, so per
your note, executables are intentionally out of scope. For everything else
(`.txt`, `.png`, `.pdf`, whatever extensions you add), Standard Lock:

1. Backs up the current default app (its registry ProgID) for that extension.
2. Points the extension at `standard-lock.exe "%1"` (per-user registry key,
   `HKCU\Software\Classes`, so **no admin rights required**).
3. When you double-click a file, Windows launches `standard-lock.exe` with the
   file's path as an argument. It checks the password rules in `config.json`;
   if the file needs a password, a small native prompt appears.
4. On the correct password, it hands the file to the original app it backed
   up in step 1 and exits. Wrong password / Cancel → nothing opens.

This is a "gate," not encryption — the file's contents on disk aren't
scrambled. If that matters for your use case, say so and I can add
transparent encrypt-on-unlock/decrypt-on-lock instead of (or alongside) the
gate.

## Per-file control (not just per-extension)

Windows associations only work at the *extension* level, so under the hood
every locked extension routes through `standard-lock.exe`. Standard Lock -
Home gives you real per-file control on top of that with two modes:

- **Lock all (default):** every file of a locked type needs the password,
  *except* ones you add to the exceptions list.
- **Lock selected only:** nothing needs the password by default; only files
  you explicitly add require one.

Toggle this with the checkbox in Home; "Select All" is simply leaving
**Lock all** checked with an empty exceptions list, which is the default.

## config.json

Lives at `%ProgramData%\StandardLock\config.json` (shared, all-users path —
deliberately *not* next to either .exe, since the two apps ship in separate
folders but must agree on one password and one file list). A template is at
the repo root: `config.example.json`. Fields:

```json
{
  "password_hash": "",          // Argon2id hash, never plaintext
  "locked_extensions": [...],   // extensions routed through Standard Lock
  "lock_all": true,             // true = lock-all-but-exceptions, false = allow-list
  "exceptions": [...],          // used when lock_all = true
  "explicit_locked_paths": [...], // used when lock_all = false
  "original_handlers": {...}    // backup so files can be handed back correctly
}
```

## Building

Requires the Rust toolchain (`rustup`) on **Windows**, since this uses native
Win32 APIs (`windows` crate) that only compile for Windows targets.

```powershell
cd standard-lock
cargo build --release
```

Produces:
- `target\release\standard-lock.exe` (~ a few MB, no GUI framework/GPU
  context — just native controls, which is what keeps it well under the
  25MB-RAM target even while the prompt is on screen)
- `target\release\standard-lock-home.exe`

Copy each into its own install folder. The release profile in the root
`Cargo.toml` (size optimization, LTO, stripped symbols) keeps both binaries
small and fast to launch — expect low single-digit MB per exe, far under the
100MB budget.

## First-time setup

1. Run `standard-lock-home.exe` once.
2. **Set a password** (top section).
3. Click **Locate standard-lock.exe...** and point it at your `lock-app`
   build — Home needs to know where it lives to register it as the file
   handler.
4. Review the locked extensions list (defaults to txt/png/jpg/jpeg/gif/pdf/
   docx) — add or remove types with the **Add Type** / **Remove Type**
   controls.
5. Leave **"Lock ALL files of these types"** checked (default / "select all"),
   or uncheck it and add specific files to lock individually instead.
6. Click **Apply Changes**. This writes `config.json` and updates the
   registry associations immediately — no reboot needed.
7. Optionally run `standard-lock.exe` once with no arguments so it sits in
   the tray; add it to your Startup folder / a registry Run key if you want
   it there automatically on login (not wired up automatically, since you
   didn't ask for it — happy to add that as a one-line registry entry if you
   want it).

## Known limitations (honest ones, not buried)

- **Per-user, not system-wide.** Because no-admin-required was implied by
  "just intercept files," associations are written to `HKCU`, so they apply
  to the Windows user account that ran Apply, not every account on the PC.
- **Modern "Store app" default handlers** (e.g. the Photos app on newer
  Windows builds) don't expose a plain command line the way classic desktop
  apps do. `open_with_original_handler` has a fallback for this (briefly
  releases the lock, opens normally, re-locks after a few seconds), but for
  the most reliable behavior, set a traditional desktop app (Notepad,
  IrfanView, a classic PDF reader, etc.) as the default for locked types.
- **Not encryption.** See the note above — it's an access gate, not a
  cryptographic guarantee. A sufficiently technical user with the file could
  still bypass it outside Explorer's normal double-click flow (e.g. opening
  it from within another running app's own "Open" dialog). Say the word if
  you want the stronger, encrypt-at-rest version instead.
- I wrote and reviewed this code carefully but **haven't compiled it** —
  this sandbox is Linux-only and the `windows` crate only targets Windows.
  Run `cargo build --release` on a Windows machine; if you hit a compiler
  error, it'll most likely be a minor `windows` crate API signature drift
  (the crate's API shifts slightly between minor versions) — paste me the
  error and I'll fix it immediately.
