// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod downloader;
mod epub_builder;
mod model;
mod secret;
mod utils;
mod config;

use config::{get_config, Config, update_config};
use downloader::Downloader;
use model::{BookInfo, Message, VolumeInfo};
use tauri::AppHandle;

#[tauri::command]
async fn get_book_info(
    app: AppHandle,
    book_id: String,
) -> Result<(BookInfo, Vec<VolumeInfo>), ()> {
    let result = tokio::task::spawn_blocking(move || {
        let config = get_config();
        Downloader::new(config.url_base.clone(), book_id, config.output_path.clone(), config.add_number, Message::new(app), config.sleep_time, &config.cookie)
    })
    .await
    .unwrap();
    if let Ok(result) = result {
        return Ok((result.book_info, result.volume_list));
    }
    Err(())
}

#[tauri::command]
async fn download(
    app: AppHandle,
    book_id: String,
    book_info: BookInfo,
    volume_list: Vec<VolumeInfo>,
    volume_no_list: Vec<usize>,
) -> Result<(), ()> {
    let _ = tokio::task::spawn_blocking(move || {
        let config = get_config();
        let downloader = Downloader::new_from(
            config.url_base.clone(),
            book_id,
            config.output_path.clone(),
            book_info,
            volume_list,
            config.add_number,
            Message::new(app),
            config.sleep_time,
            &config.cookie
        );
        downloader.download(volume_no_list.into_iter())
    })
    .await;
    Ok(())
}

#[tauri::command]
async fn save_config(config: Config) -> Result<(), ()> {
    update_config(config);
    match config::save_config() {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

#[tauri::command]
async fn get_config_vue() -> Config {
    get_config().clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_book_info, download, save_config, get_config_vue])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}