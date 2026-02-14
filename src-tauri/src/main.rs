// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use epub_download_lib::runtime::{RUN_MODE, RunMode};

#[cfg(windows)]
fn attach_console() {
    use windows::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};

    unsafe {
        // 把当前进程附加到父控制台
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}
#[tokio::main]
async fn main() {
    #[cfg(windows)]
    attach_console();

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        // CLI mode
        *RUN_MODE.lock() = RunMode::Cli;
        epub_download_lib::run_cli().await.unwrap();
    } else {
        *RUN_MODE.lock() = RunMode::Gui;
        #[cfg(feature = "gui")]
        epub_download_lib::run()
    }
}
