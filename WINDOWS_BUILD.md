# Building Digital Product Factory for Windows

This is a native Rust/egui desktop app. The reliable way to get a Windows
`.exe` is to compile it on Windows (either your own machine or a free
GitHub Actions runner) — cross-compiling from Linux is fragile for this
project because several dependencies (tray-icon, keyring, rfd file dialogs,
notify-rust) link directly against the real Windows SDK.

## Option A — Automatic build via GitHub Actions (no Windows machine needed)

A workflow is already included at `.github/workflows/windows-build.yml`.

1. Push this project to a GitHub repo (public or private).
2. GitHub will automatically run the workflow on push to `main`, or you can
   trigger it manually from the Actions tab ("Run workflow").
3. When it finishes (a few minutes), open the run, and download the
   `digital-product-factory-windows` artifact — it contains `dpf.exe`.
4. Run `dpf.exe` on any Windows 10/11 machine. No install needed.

This produces a genuine native Windows binary, built by a real Windows
machine, so all the Windows-specific integrations (system tray, credential
manager, native file dialogs, notifications) work correctly.

## Option B — Build locally on a Windows PC

1. Install Rust: https://rustup.rs (downloads `rustup-init.exe`, click
   through the default install).
2. Install "Desktop development with C++" via the Visual Studio Build
   Tools if prompted (needed for linking).
3. Open PowerShell in this project folder and run:
   ```
   cargo build --release
   ```
4. The executable will be at `target\release\dpf.exe`.

## Notes

- The `Cargo.toml` already targets a single binary named `dpf`, so the
  output name is the same on Windows (`dpf.exe`).
- `rusqlite` is set to `bundled`, so SQLite compiles in automatically —
  no separate SQLite install needed on Windows.
- `reqwest` is configured with `rustls-tls`, so no OpenSSL install is
  needed either.
- First build will take a while (many dependencies, LTO enabled in the
  release profile). Subsequent builds are much faster.
