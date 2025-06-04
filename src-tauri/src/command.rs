use crate::config::Config;
use crate::downloader::Downloader;
use crate::error::Result;
use crate::model::{BookInfo, Message, VolumeInfo};
use parking_lot::RwLock;
use tauri::{AppHandle, State};

#[tauri::command]
#[specta::specta]
pub async fn get_book_info(
    config: State<'_, RwLock<Config>>,
    app: AppHandle,
    book_id: String,
) -> Result<(BookInfo, Vec<VolumeInfo>)> {
    let (base_url, output, template, sleep_time, cookie, add_catalog, error_img) = {
        let config = config.read();

        (
            config.base_url.clone(),
            config.output.clone(),
            config.template.clone(),
            config.sleep_time,
            config.cookie.clone(),
            config.add_catalog,
            config.error_img.clone(),
        )
    }; // config 在这里自动 drop 释放锁

    let handle = tokio::task::spawn_blocking(move || {
        let result = Downloader::new(
            base_url,
            book_id,
            output,
            template,
            Message::new(Some(app)),
            sleep_time,
            &cookie,
            add_catalog,
            error_img,
        )?;
        Ok::<_, anyhow::Error>((result.book_info, result.volume_list))
    });

    let result = handle.await??;

    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub async fn download(
    config: State<'_, RwLock<Config>>,
    app: AppHandle,
    book_id: String,
    book_info: BookInfo,
    volume_list: Vec<VolumeInfo>,
    volume_no_list: Vec<u32>,
) -> Result<()> {
    let (base_url, output, template, sleep_time, cookie, add_catalog, error_img) = {
        let config = config.read();
        (
            config.base_url.clone(),
            config.output.clone(),
            config.template.clone(),
            config.sleep_time,
            config.cookie.clone(),
            config.add_catalog,
            config.error_img.clone(),
        )
    };

    let message = Message::new(Some(app));

    let handle = tokio::task::spawn_blocking(move || {
        let downloader = Downloader::new_from(
            base_url,
            book_id,
            output,
            book_info,
            volume_list,
            template,
            message,
            sleep_time,
            &cookie,
            add_catalog,
            error_img,
        );
        downloader.download(volume_no_list.into_iter())
    });

    let result = handle.await??;

    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub async fn save_config(config: State<'_, RwLock<Config>>, new_config: Config) -> Result<()> {
    let mut config = config.write();
    *config = new_config;
    Ok(config.save()?)
}

#[tauri::command]
#[specta::specta]
pub async fn get_config_vue(config: State<'_, RwLock<Config>>) -> Result<Config> {
    let config = config.read();
    Ok(config.clone())
}
