use crate::config::Config;
use crate::downloader::Downloader;
use crate::error::Result;
use crate::model::{Book, BookInfo, VolumeInfo};
use parking_lot::RwLock;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::broadcast;

// 全局取消通道类型
pub type CancelSender = Arc<broadcast::Sender<()>>;

#[tauri::command]
#[specta::specta]
pub async fn get_book_info(
    config: State<'_, RwLock<Config>>,
    app: AppHandle,
    book_id: String,
) -> Result<(BookInfo, Vec<VolumeInfo>)> {
    let (base_url, output, template, sleep_time, cookie, user_agent, add_catalog, error_img) = {
        let config = config.read();

        (
            config.base_url.clone(),
            config.output.clone(),
            config.template.clone(),
            config.sleep_time,
            config.cookie.clone(),
            config.user_agent.clone(),
            config.add_catalog,
            config.error_img.clone(),
        )
    }; // config 在这里自动 drop 释放锁

    let result = Downloader::new(
        base_url,
        book_id,
        output,
        template,
        sleep_time,
        &cookie,
        &user_agent,
        add_catalog,
        error_img,
        Some(app),
    )
    .await?;

    let result = (result.book_info, result.volume_infos);

    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub async fn download(
    config: State<'_, RwLock<Config>>,
    cancel_sender: State<'_, CancelSender>,
    app: AppHandle,
    book_id: String,
    book_info: BookInfo,
    volume_list: Vec<VolumeInfo>,
    volume_no_list: Vec<u32>,
) -> Result<()> {
    let (base_url, output, template, sleep_time, cookie, user_agent, add_catalog, error_img) = {
        let config = config.read();
        (
            config.base_url.clone(),
            config.output.clone(),
            config.template.clone(),
            config.sleep_time,
            config.cookie.clone(),
            config.user_agent.clone(),
            config.add_catalog,
            config.error_img.clone(),
        )
    };

    // 创建取消接收器
    let mut cancel_receiver = cancel_sender.subscribe();

    // 使用 tokio::select! 来处理下载任务和取消信号
    tokio::select! {
        result = async {
            let downloader = Downloader::new_from(
                base_url,
                book_id,
                output,
                book_info,
                volume_list,
                template,
                sleep_time,
                &cookie,
                &user_agent,
                add_catalog,
                error_img,
                Some(app),
            )?;
            downloader.download(volume_no_list.into_iter()).await
        } => {
            result?;
        }
        _ = cancel_receiver.recv() => {
            return Ok(()); // 收到取消信号，正常返回
        }
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn cancel_download(cancel_sender: State<'_, CancelSender>) -> Result<()> {
    let _ = cancel_sender.send(());
    Ok(())
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

#[tauri::command]
#[specta::specta]
pub async fn check_update() -> Result<String> {
    let client = crate::client::BiliClient::new("https://www.bilinovel.com", "", "")?;
    let result = client.check_update().await?;
    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub async fn get_version() -> Result<&'static str> {
    let result = env!("CARGO_PKG_VERSION");
    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub async fn get_books() -> Result<Vec<Book>> {
    let books = crate::manage::get_books(crate::config::INDEX_FILE)?;
    Ok(books)
}

#[tauri::command]
#[specta::specta]
pub async fn create_index(config: State<'_, RwLock<Config>>) -> Result<()> {
    let config = config.read();
    crate::manage::create_index(&config.output, crate::config::INDEX_FILE)?;
    Ok(())
}
