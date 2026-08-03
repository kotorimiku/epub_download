use crate::{error::Result, utils};

pub async fn check_update() -> Result<String> {
    let url = "https://api.github.com/repos/kotorimiku/epub_download/releases/latest";
    let res = reqwest::Client::new().get(url).send().await?;
    let json = res.json::<serde_json::Value>().await?;
    let version = match json["tag_name"].as_str() {
        Some(v) => v,
        None => return Ok("未获取到最新版本号".into()),
    };
    let local_version = env!("CARGO_PKG_VERSION");
    let is_newer = utils::is_newer_version(local_version, version);
    let download_url = match json["html_url"].as_str() {
        Some(url) => url,
        None => return Ok("未获取到下载地址".into()),
    };
    if is_newer {
        Ok(format!("最新版本: {}\n下载地址: {}", version, download_url))
    } else {
        Ok(format!("已是最新版本: {}", version))
    }
}
