use anyhow::anyhow;
use anyhow::Result;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, COOKIE, USER_AGENT};

use crate::model::Message;
use crate::utils;
use crate::utils::t2s;

pub fn get_headers(referer: &str, cookie: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Linux; Android 11; M2102J20SG) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.99 Mobile Safari/537.36 EdgA/97.0.1072.78"));
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
    base_url: String,
}

impl BiliClient {
    pub fn new(referer: &str, cookie: &str) -> Self {
        let headers = get_headers(referer, cookie);
        Self {
            client: Client::builder().default_headers(headers).build().unwrap(),
            base_url: referer.to_string(),
        }
    }

    pub fn get_html(&self, url: &str, message: &Message, sleep_time: u32) -> Result<String> {
        println!("  {url}");

        std::thread::sleep(std::time::Duration::from_secs(sleep_time.into()));
        if let Ok(res) = self.client.get(url).send() {
            if res.url().as_str() != url {
                message.send("url重定向");
                message.send(&format!("原始url: {}", url));
                message.send(&format!("重定向到: {}", res.url()));
                return Err(anyhow!("url重定向"));
            }
            if let Ok(t) = res.text() {
                let mut text = t;
                if url.contains("tw.linovelib.com") {
                    text = t2s(&text);
                }
                if text.contains("used Cloudflare to restrict access") {
                    message.send("下载频繁，触发反爬，正在重试....");
                    std::thread::sleep(std::time::Duration::from_secs(10));
                    return self.get_html(url, message, sleep_time);
                }
                if text.contains("Just a moment...") || text.contains("403 Forbidden") {
                    message.send("下载失败，请稍后再试");
                    return Err(anyhow!("下载失败，请稍后再试"));
                }
                if text.contains("對不起，該書內容已刪除")
                    || text.contains("对不起，该书内容已删除")
                {
                    message.send("该书内容已删除");
                    return Err(anyhow!("该书内容已删除"));
                }
                if text.contains("章節內容審核未通過") || text.contains("章节内容审核未通过")
                {
                    message.send("该书内容审核未通过");
                    return Err(anyhow!("该书内容审核未通过"));
                }
                if text.contains("通告～客戶端停用中")
                    || text.contains("通告～客户端停用中")
                    || text.contains("內容加载失败")
                    || text.contains("手机版页面由于相容性问题暂不支持电脑端阅读")
                {
                    message.send("无法下载完整内容，正在重试....");
                    std::thread::sleep(std::time::Duration::from_secs(10));
                    return self.get_html(url, message, sleep_time);
                }
                return Ok(text);
            }
        }
        message.send("请求失败，正在重试....");
        std::thread::sleep(std::time::Duration::from_secs(3));
        self.get_html(url, message, sleep_time)
    }

    pub fn get_novel(&self, book_id: &str, message: &Message) -> Result<String> {
        let url = &format!("{}/novel/{}.html", self.base_url, book_id);

        self.get_html(url, message, 0)
    }

    pub fn get_volume_catalog(&self, book_id: &str, message: &Message) -> Result<String> {
        let url = &format!("{}/novel/{}/catalog", self.base_url, book_id);

        self.get_html(url, message, 0)
    }

    pub fn get_img_bytes(&self, url: &str) -> Result<Vec<u8>> {
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
        if let Ok(response) = client.send() {
            // let length = response.content_length().unwrap_or(0);
            // let code = response.status().as_u16();
            let data = response.bytes().unwrap();

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
}
