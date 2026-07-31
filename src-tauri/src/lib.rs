pub use bilinovel as core;
pub use bilinovel::{BiliClient, BiliNovel};

pub mod cli;
pub mod config;
pub mod downloader;
pub mod epub_builder;
pub mod error;
pub mod manage;
pub mod message;
pub mod runtime;

pub use cli::run_cli;
pub use config::Config;
pub use error::{CommandError, Result};

#[cfg(feature = "gui")]
pub mod command;
#[cfg(feature = "gui")]
pub mod event;

#[cfg(feature = "gui")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use std::{collections::HashMap, sync::Arc};

    use parking_lot::RwLock;
    use tauri_specta::{Builder, collect_commands};
    use tokio::sync::broadcast;

    use crate::{command::*, config::Config};

    // 创建取消通道
    let (cancel_sender, _) = broadcast::channel::<()>(1);
    let cancel_sender = Arc::new(cancel_sender);
    let js_cache: JsCache = Arc::new(RwLock::new(HashMap::new()));

    let builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            get_book_info,
            download,
            cancel_download,
            browser_url,
            fetch_js,
            save_config,
            get_config_vue,
            check_update,
            get_version,
            get_books,
            create_index,
            request_img,
        ])
        .error_handling(tauri_specta::ErrorHandlingMode::Throw);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    let config = Config::load();
    let client = BiliClient::new(
        &config.base_url,
        &config.cookie,
        &config.user_agent,
        &config.headers,
        config.convert_simple_chinese,
        config.debug,
    )
    .expect("Failed to initialize BiliClient");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .manage(RwLock::new(config))
        .manage(RwLock::new(client))
        .manage(cancel_sender)
        .manage(js_cache)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
