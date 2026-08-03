use crate::{error::Result, utils};

pub async fn check_update() -> Result<String> {
    let cdn_url = "https://cdn.jsdelivr.net/gh/kotorimiku/epub_download@main/package.json";
    let raw_url = "https://raw.githubusercontent.com/kotorimiku/epub_download/main/package.json";

    let client = reqwest::Client::new();
    let res = match client.get(cdn_url).send().await {
        Ok(resp) if resp.status().is_success() => resp,
        _ => client.get(raw_url).send().await?,
    };

    let json = res.json::<serde_json::Value>().await?;
    let version = match json["version"].as_str() {
        Some(v) => v,
        None => return Ok("未获取到最新版本号".into()),
    };

    let local_version = env!("CARGO_PKG_VERSION");
    let is_newer = utils::is_newer_version(local_version, version);
    let download_url = "https://github.com/kotorimiku/epub_download/releases/latest";

    if is_newer {
        Ok(format!("最新版本: {}\n下载地址: {}", version, download_url))
    } else {
        Ok(format!("已是最新版本: {}", version))
    }
}

