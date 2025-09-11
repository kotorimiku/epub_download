// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod client;
pub mod config;
pub mod downloader;
pub mod epub_builder;
pub mod error;
pub mod manage;
pub mod message;
pub mod model;
pub mod paragraph_restorer;
pub mod parse;
pub mod secret;
pub mod utils;

#[cfg(feature = "gui")]
pub mod command;
#[cfg(feature = "gui")]
pub mod event;

#[cfg(feature = "gui")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use crate::command::*;
    use crate::config::Config;
    use parking_lot::RwLock;
    use tauri_specta::{collect_commands, Builder};

    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        get_book_info,
        download,
        save_config,
        get_config_vue,
        check_update,
        get_books,
        create_index,
    ]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .manage(RwLock::new(Config::load()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
