//! L3 desktop app: egui immediate-mode GUI over `arr-core`, in-process.
//!
//! ArtCraft native stack -- no WebView2, no JavaScript, one `cargo` toolchain.
//! `--smoke` runs the headless parity check (for CI) without opening a window.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::io::Write;
use std::path::PathBuf;

mod app;
mod registry;
mod vault;

fn install_forensic_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("[PANIC in arr-app]: {info}\n");
        let _ = std::io::stderr().write_all(msg.as_bytes());
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let dir = PathBuf::from(local).join("rust-arr-mcp");
            let _ = std::fs::create_dir_all(&dir);
            let log_file = dir.join("crash.log");
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)
            {
                let _ = f.write_all(msg.as_bytes());
            }
        }
    }));
}

fn main() -> eframe::Result<()> {
    install_forensic_panic_hook();
    if std::env::args().any(|arg| arg == "--smoke") {
        return app::smoke();
    }
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => std::sync::Arc::new(rt),
        Err(e) => {
            eprintln!("failed to start async runtime: {e}");
            std::process::exit(1);
        }
    };
    let viewport = egui::ViewportBuilder::default()
        .with_title("Arr Stack (Rust) v0.1")
        .with_inner_size([1320.0, 860.0])
        .with_min_inner_size([920.0, 600.0]);
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Arr Stack (Rust)",
        options,
        Box::new(move |ctx| Ok(Box::new(app::ArrApp::new(ctx, runtime)))),
    )
}
