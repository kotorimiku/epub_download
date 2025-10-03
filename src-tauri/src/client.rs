use anyhow::anyhow;
use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, COOKIE, USER_AGENT};
use reqwest::Client;
use url::Url;

use crate::message::send;
use crate::model::App;
use crate::utils;
use crate::utils::t2s;

pub fn get_headers(referer: &str, mut cookie: &str, mut user_agent: &str) -> HeaderMap {
    if cookie.is_empty() {
        cookie = "cf_clearance=MhyWh1HcQwVSfqQ0YuGDHdGQV7byVnrStRtUluyvW5A-1742750359-1.2.1.1-mjTA91HfTFNjK9av6_GY7SqPux3rniOzJiA..fTOkzzgLbr_sNQQCQkcZXXgi9Yo4ML_VHTOsqmt4WYXRFAWwVuTESUsupsJfkV9k6MamwEJcnUL5pvpcu2Vn0H2fQEdzenu8htt8qxXUAcA0GnTI95CLvND3tjbFVWGuMg6BEXZZ9gWKncAMNIM4Oajs4faI6YV3hvrOtZOL5NcWa25cyBXbQmRvyQWn5v1UH5xszIFZ87VRSotm9ehiXiiodXmyBzXlzZR48sa4uP2nfsPp1FFJIsCsGvr7m1XH2eD7zmqdY48qOQQjxzcRAJ8qZK27lx1mnn8n3Wmse54R6Q44j9XxmvWFurmV_xh3gmVW6XP01sTEp1Aua.8JRiqTPSm5xbicJKkM3pXkyUxnBMOBIGmUzw2MghJwV4SNps.2aw; jieqiVisitInfo=jieqiUserLogin%3D1747807212%2CjieqiUserId%3D220564; night=1; cf_chl_rc_m=1; cf_clearance=M5TuaQ_ycgp5WRpEeF3datJC1PXAbk2PFJIX1Q0JsMQ-1757599333-1.2.1.1-BQSxNonxHm4oItlzo9FeFiZE52V_3.NtLTFmX1EBQWkuYbrTmWcR.k_Un6T5Tsy2CgP9rTJNZDOb8xeQvUi4DG8HNVwS.F6.vb1fpa3y.k2OeeLQlURXeB4apx9xUVL57kYgE3wz2qHnYkjZ2ekVd9MkWF2tdrBVG14oTKkDMcQiK7Kp_XfPodu1G90NHiVyRm5pagD1my2LqIDiby8bnNftG0sPSj6n8hLF6OR5blJsDEjkeSlyluj4sRpruCRv"
    }
    if user_agent.is_empty() {
        user_agent = "Mozilla/5.0 (Linux; U; Android 4.0.2; en-us; Galaxy Nexus Build/ICL53F) AppleWebKit/534.30 (KHTML, like Gecko) Version/4.0 Mobile Safari/534.30";
    }
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_str(user_agent).unwrap());
    headers.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_static(
            "en,zh-HK;q=0.9,zh-TW;q=0.8,zh-CN;q=0.7,zh;q=0.6,en-GB;q=0.5,en-US;q=0.4",
        ),
    );
    headers.insert(ACCEPT, HeaderValue::from_static(r"text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"));
    headers.insert(COOKIE, HeaderValue::from_str(cookie).unwrap());
    headers.insert(
        "Referer",
        HeaderValue::from_str(&(referer.to_string() + "/novel/4353/250879.html")).unwrap(),
    );
    headers.insert(
        "accept-encoding",
        HeaderValue::from_static("gzip, deflate, br, zstd"),
    );
    headers.insert("priority", HeaderValue::from_static("u=0, i"));
    headers.insert(
        "sec-ch-ua",
        HeaderValue::from_static(
            "\"Microsoft Edge\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"",
        ),
    );
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?1"));
    headers.insert(
        "sec-ch-ua-platform",
        HeaderValue::from_static("\"Android\""),
    );
    headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
    headers.insert("upgrade-insecure-requests", HeaderValue::from_static("1"));
    headers.insert("sec-fetch-user", HeaderValue::from_static("?1"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers
}

pub struct BiliClient {
    client: Client,
    base_url: Url,
}

impl BiliClient {
    pub fn new(referer: &str, cookie: &str, user_agent: &str) -> Result<Self> {
        let headers = get_headers(referer, cookie, user_agent);
        Ok(Self {
            client: Client::builder().default_headers(headers).build()?,
            base_url: Url::parse(referer)?,
        })
    }

    pub async fn get(&self, url: &str) -> Result<String> {
        if let Ok(res) = self.client.get(url).send().await {
            return res.text().await.map_err(|e| anyhow!(e));
        } else {
            return Err(anyhow!("请求失败"));
        }
    }

    pub async fn get_html(
        &self,
        url: &str,
        message: &Option<App>,
        sleep_time: u32,
    ) -> Result<String> {
        println!("  {url}");

        tokio::time::sleep(std::time::Duration::from_secs(sleep_time.into())).await;

        loop {
            if let Ok(res) = self.client.get(url).send().await {
                if res.url().as_str() != url {
                    send(message, "url重定向");
                    send(message, &format!("原始url: {}", url));
                    send(message, &format!("重定向到: {}", res.url()));
                    return Err(anyhow!("url重定向"));
                }
                if let Ok(t) = res.text().await {
                    let mut text = t;
                    if url.contains("tw.linovelib.com") {
                        text = t2s(&text);
                    }
                    if text.contains("used Cloudflare to restrict access") {
                        send(message, "下载频繁，触发反爬，正在重试....");
                        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                        continue; // 重试
                    }
                    if text.contains("Just a moment...") || text.contains("403 Forbidden") {
                        send(message, "下载失败，请稍后再试");
                        return Err(anyhow!("下载失败，请稍后再试"));
                    }
                    if text.contains("對不起，該書內容已刪除")
                        || text.contains("对不起，该书内容已删除")
                    {
                        send(message, "该书内容已删除");
                        return Err(anyhow!("该书内容已删除"));
                    }
                    if text.contains("章節內容審核未通過") || text.contains("章节内容审核未通过")
                    {
                        send(message, "该书内容审核未通过");
                        return Err(anyhow!("该书内容审核未通过"));
                    }
                    if text.contains("抱歉，该小说未经审核")
                        || text.contains("抱歉，該小說未經審核")
                    {
                        send(message, "该小说未经审核");
                        return Err(anyhow!("该小说未经审核"));
                    }
                    if text.contains("抱歉，該小說不存在") || text.contains("抱歉，该小说不存在")
                    {
                        send(message, "该小说不存在");
                        return Err(anyhow!("该小说不存在"));
                    }
                    if text.contains("通告～客戶端停用中")
                        || text.contains("通告～客户端停用中")
                        || text.contains("內容加载失败")
                        || text.contains("手机版页面由于相容性问题暂不支持电脑端阅读")
                    {
                        send(message, "无法下载完整内容，正在重试....");
                        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                        continue; // 重试
                    }
                    return Ok(text);
                }
            }
            send(message, "请求失败，正在重试....");
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            // 继续循环重试
        }
    }

    pub async fn get_novel(&self, book_id: &str, message: &Option<App>) -> Result<String> {
        let url = self.base_url.join(&format!("/novel/{}.html", book_id))?;

        self.get_html(url.as_str(), message, 0).await
    }

    pub async fn get_volume(
        &self,
        book_id: &str,
        volume_id: &str,
        message: &Option<App>,
    ) -> Result<String> {
        let url = self
            .base_url
            .join(&format!("/novel/{}/vol_{}.html", book_id, volume_id))?;

        self.get_html(url.as_str(), message, 0).await
    }

    pub async fn get_catalog(&self, book_id: &str, message: &Option<App>) -> Result<String> {
        let url = self.base_url.join(&format!("/novel/{}/catalog", book_id))?;

        self.get_html(url.as_str(), message, 0).await
    }

    pub async fn get_img_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let mut client = self.client.get(url).header(
            ACCEPT,
            "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8",
        );
        if url.contains("masiro") {
            client = client.header(
                "Referer",
                HeaderValue::from_static("https://www.masiro.me/"),
            );
        }
        if let Ok(response) = client.send().await {
            // let length = response.content_length().unwrap_or(0);
            // let code = response.status().as_u16();
            let data = response.bytes().await?;

            // if code == 404 {
            //     self.message
            //         .send(&format!("\n  插图下载失败，404 Not Found {}", img_url));
            //     return Vec::new();
            // }

            // if length != data.len() as u64 {
            //     return Err("插图下载失败".to_string());
            // }

            return match utils::img_to_jpg(data.to_vec()) {
                Ok(data) => Ok(data),
                Err(_) => {
                    return Err(anyhow!("插图下载失败"));
                }
            };
        }
        return Err(anyhow!("插图下载失败"));
    }

    pub async fn check_update(&self) -> Result<String> {
        let url = "https://api.github.com/repos/kotorimiku/epub_download/releases/latest";
        let res = self.client.get(url).send().await?;
        let json = res.json::<serde_json::Value>().await?;
        let version = json["tag_name"]
            .as_str()
            .ok_or_else(|| anyhow!("未获取到最新版本号"))?;
        let local_version = env!("CARGO_PKG_VERSION");
        let is_newer = utils::is_newer_version(local_version, version);
        let download_url = json["html_url"]
            .as_str()
            .ok_or_else(|| anyhow!("未获取到下载地址"))?;
        if is_newer {
            Ok(format!("最新版本: {}\n下载地址: {}", version, download_url))
        } else {
            Ok(format!("已是最新版本: {}", version))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn download_test() {
        let client = BiliClient::new("https://www.bilinovel.com", "", "");
        let result = client
            .unwrap()
            .get("https://www.bilinovel.com/novel/115/catalog")
            .await
            .unwrap();
        println!("{}", result);
    }
}
