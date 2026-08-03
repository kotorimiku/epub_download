use std::{collections::HashMap, sync::Arc};

use bilinovel::{BiliClient, Book, BookInfo, VolumeInfo};
use parking_lot::RwLock;
use tauri::{AppHandle, State, ipc::Channel};
use tokio::sync::broadcast;

use crate::{
    config::Config,
    downloader::{Downloader, DownloaderConfig},
    error::CommandError,
};

type Result<T> = std::result::Result<T, CommandError>;

// 全局取消通道类型
pub type CancelSender = Arc<broadcast::Sender<()>>;

#[tauri::command]
#[specta::specta]
pub async fn get_book_info(
    config: State<'_, RwLock<Config>>,
    client: State<'_, RwLock<BiliClient>>,
    app: AppHandle,
    book_id: String,
) -> Result<(BookInfo, Vec<VolumeInfo>)> {
    let downloader_config = {
        let config = config.read();
        let client = client.read().clone();
        DownloaderConfig::new(&config, book_id, Some(app)).with_client(client)
    }; // config 在这里自动 drop 释放锁

    let result = Downloader::new(downloader_config).await?;

    let result = (result.book_info, result.volume_infos);

    Ok(result)
}

#[tauri::command]
#[specta::specta]
#[allow(clippy::too_many_arguments)]
pub async fn download(
    config: State<'_, RwLock<Config>>,
    client: State<'_, RwLock<BiliClient>>,
    cancel_sender: State<'_, CancelSender>,
    app: AppHandle,
    book_id: String,
    book_info: BookInfo,
    volume_list: Vec<VolumeInfo>,
    volume_no_list: Vec<u32>,
) -> Result<()> {
    let downloader_config = {
        let config = config.read();
        let client = client.read().clone();
        DownloaderConfig::new(&config, book_id, Some(app)).with_client(client)
    };

    // 创建取消接收器
    let mut cancel_receiver = cancel_sender.subscribe();

    // 使用 tokio::select! 来处理下载任务和取消信号
    tokio::select! {
        result = async {
            let downloader = Downloader::new_from(downloader_config, book_info, volume_list)?;
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

pub type JsCache = Arc<RwLock<HashMap<String, String>>>;

#[tauri::command]
#[specta::specta]
pub async fn fetch_js(
    url: String,
    client: State<'_, RwLock<BiliClient>>,
    cache: State<'_, JsCache>,
) -> Result<String> {
    if let Some(content) = cache.read().get(&url) {
        return Ok(content.clone());
    }

    let client = client.read().clone();
    let result = client.get(&url).await?;
    if result.is_empty() || result.contains("Just a moment") {
        return Err(CommandError(
            "Failed to fetch JS content, possibly due to anti-scraping measures.".to_string(),
        ));
    }
    cache.write().insert(url, result.clone());
    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub async fn browser_url(url: String, client: State<'_, RwLock<BiliClient>>) -> Result<String> {
    let client = client.read().clone();
    let result = client.get(&url).await?;
    Ok(result)
}

#[derive(Debug, Clone, specta::Type, serde::Serialize, serde::Deserialize)]
pub enum Tls {
    NativeTls,
    Rustls,
}

#[tauri::command]
#[specta::specta]
pub async fn request_img(
    url: String,
    channel: Channel<Vec<u8>>,
    client: State<'_, RwLock<BiliClient>>,
) -> Result<()> {
    let client = client.read().clone();
    let result = client.get_img_bytes(&url, None::<&fn(&str)>).await?;
    channel.send(result)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn save_config(
    config: State<'_, RwLock<Config>>,
    client: State<'_, RwLock<BiliClient>>,
    new_config: Config,
) -> Result<()> {
    let new_client = bilinovel::BiliClient::new(
        &new_config.base_url,
        &new_config.cookie,
        &new_config.user_agent,
        &new_config.headers,
        new_config.convert_simple_chinese,
        new_config.debug,
    )?;
    {
        let mut config = config.write();
        *config = new_config;
        config.save()?;
    }
    {
        let mut client = client.write();
        *client = new_client;
    }
    Ok(())
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
    let result = crate::client::check_update().await?;
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
