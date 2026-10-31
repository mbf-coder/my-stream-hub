//! MY STREAM HUB — desktop entrypoint (Phase 1).
//!
//! On Windows (default features) this boots the Tauri 2 runtime.
//! Verification builds without system WebKit (`--no-default-features`) compile
//! to a stub binary that reports the configuration instead of opening a window.

#[cfg(feature = "tauri-app")]
fn main() {
    my_stream_hub::run_app();
}

#[cfg(not(feature = "tauri-app"))]
fn main() {
    println!("my-stream-hub core build (no Tauri shell). Run with default features on Windows 11.");
}
