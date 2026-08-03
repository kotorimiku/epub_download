use std::collections::HashMap;

use reqwest::{
    Client,
    header::{ACCEPT, ACCEPT_LANGUAGE, COOKIE, HeaderMap, HeaderName, HeaderValue, USER_AGENT},
};
use url::Url;

use crate::{
    bail,
    error::Result,
    listener::MessageCallback,
    utils::{self, t2s},
};

fn send_msg(callback: Option<&impl MessageCallback>, msg: &str) {
    if let Some(cb) = callback {
        cb(msg);
    }
    println!("{}", msg);
}

pub fn get_headers(
    referer: &str,
    mut cookie: &str,
    mut user_agent: &str,
    header_map: &HashMap<String, String>,
) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();

    if !header_map.is_empty() {
        for (k, v) in header_map {
            if let (Ok(name), Ok(value)) = (
                HeaderName::from_bytes(k.as_bytes()),
                HeaderValue::from_str(v),
            ) {
                headers.insert(name, value);
            }
        }

        return Ok(headers);
    }

    if cookie.is_empty() {
        cookie = "cf_clearance=HF9zkHeiCy3K5h31HgAdgA.eLYiEiGHp8HyAys8U3bY-1785133887-1.2.1.1-2jwMdr9ZFCPexWj8qB.71IbRRuPnxVxqABELCjFdTmhpZjg4hvKhpPVn39ttPejeWl.z73wDO1BtVHvG.PuIpSlVFz_i4eLxVqPV9rTCVYmp2eBzg7Zj8_yebYv4.kD4HJSdOIdWrnutiGG4yK63dXUSY6AqBUvpv9YgSdqC_WCCI5n8fuw.1jyawYtnn5GHG5Cy6VafhX3_60BPrlJbuLGTw3tv61WNvktcYP0Aufj0drF5qktHJPfThx_5ajwfLldiuscUizDjGY.sD9GPioMShSAWUPIIL3T5BehJSWjduDg7VJWLaP2ghlUN1JYIjcSImtwDby2QSMG8PQGqRuZwPNZidKpSlplsaE10R4eDNRH81kDtq3LYcSHjrOpBlKAyWaJMK9MFJpzZOlEY_NfeVGaMoYIWpjxwKbJaf.gHaQtSMHT3bmZVadad_m8fzQ32Jk3crF5I4D1URDy1RfCQXZiqFFMeKTKSUzWa1hFWWSwZxDbXmITJqflmqjHhdymQ1b593_3iMosBJJtHq5e460jLP4JVmWzSjoFFJHJdiqwgKi_UUaB8z8wVoe6W5Lm951jwREXrfHdVMr_ywA; night=0; user_tz=Asia%2FShanghai";
    }
    if user_agent.is_empty() {
        user_agent = "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36";
    }
    headers.insert(USER_AGENT, HeaderValue::from_str(user_agent.trim())?);
    headers.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_static(
            "en,zh-HK;q=0.9,zh-TW;q=0.8,zh-CN;q=0.7,zh;q=0.6,en-GB;q=0.5,en-US;q=0.4",
        ),
    );
    headers.insert(ACCEPT, HeaderValue::from_static(r"text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"));
    headers.insert(COOKIE, HeaderValue::from_str(cookie.trim())?);
    headers.insert(
        "Referer",
        HeaderValue::from_str(&(referer.to_string() + "/novel/4353/250879.html"))?,
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
    headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
    headers.insert("sec-fetch-user", HeaderValue::from_static("?1"));
    headers.insert("upgrade-insecure-requests", HeaderValue::from_static("1"));

    Ok(headers)
}

#[derive(Clone)]
pub struct BiliClient {
    client: Client,
    pub base_url: Url,
    convert_simple_chinese: bool,
    debug: bool,
}

impl BiliClient {
    pub fn new(
        base_url: &str,
        cookie: &str,
        user_agent: &str,
        header_map: &HashMap<String, String>,
        convert_simple_chinese: bool,
        debug: bool,
    ) -> Result<Self> {
        let headers = get_headers(base_url, cookie, user_agent, header_map)?;
        let client = Client::builder()
            .default_headers(headers)
            .cookie_store(true)
            .build()?;
        let base_url = Url::parse(base_url)?;
        Ok(Self {
            client,
            base_url,
            convert_simple_chinese,
            debug,
        })
    }

    pub async fn get(&self, url: &str) -> Result<String> {
        let url = if !url.starts_with("http") {
            &format!("{}{}", self.base_url, url).as_str().to_string()
        } else {
            url
        };

        if let Ok(res) = self.client.get(url).send().await {
            Ok(res.text().await?)
        } else {
            bail!("请求失败")
        }
    }

    pub async fn get_html(
        &self,
        url: &str,
        on_message: Option<&impl MessageCallback>,
        sleep_time: u32,
    ) -> Result<String> {
        let url = if !url.starts_with("http") {
            &format!("{}{}", self.base_url, url).as_str().to_string()
        } else {
            url
        };

        println!("  {url}");

        tokio::time::sleep(std::time::Duration::from_secs(sleep_time.into())).await;

        loop {
            if let Ok(res) = self.client.get(url).send().await {
                if res.url().as_str() != url {
                    send_msg(on_message, "url重定向");
                    send_msg(on_message, &format!("原始url: {}", url));
                    send_msg(on_message, &format!("重定向到: {}", res.url()));
                    bail!("url重定向");
                }
                if let Ok(t) = res.text().await {
                    let mut text = t;
                    if self.convert_simple_chinese {
                        text = t2s(&text);
                    }
                    if text.contains("used Cloudflare to restrict access") {
                        send_msg(on_message, "下载频繁，触发反爬，正在重试....");
                        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                        continue;
                    }
                    if text.contains("Just a moment...") || text.contains("403 Forbidden") {
                        send_msg(on_message, "下载失败，请稍后再试");
                        bail!("下载失败，请稍后再试");
                    }
                    if text.contains("對不起，該書內容已刪除")
                        || text.contains("对不起，该书内容已删除")
                    {
                        send_msg(on_message, "该书内容已删除");
                        bail!("该书内容已删除");
                    }
                    if text.contains("章節內容審核未通過") || text.contains("章节内容审核未通过")
                    {
                        send_msg(on_message, "该书内容审核未通过");
                        bail!("该书内容审核未通过");
                    }
                    if text.contains("抱歉，该小说未经审核")
                        || text.contains("抱歉，該小說未經審核")
                    {
                        send_msg(on_message, "该小说未经审核");
                        bail!("该小说未经审核");
                    }
                    if text.contains("抱歉，該小說不存在") || text.contains("抱歉，该小说不存在")
                    {
                        send_msg(on_message, "该小说不存在");
                        bail!("该小说不存在");
                    }
                    if text.contains("通告～客戶端停用中")
                        || text.contains("通告～客户端停用中")
                        || text.contains("內容加载失败")
                        || text.contains("手机版页面由于相容性问题暂不支持电脑端阅读")
                    {
                        send_msg(on_message, "无法下载完整内容，正在重试....");
                        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                        continue;
                    }
                    return Ok(text);
                }
            }
            send_msg(on_message, "请求失败，正在重试....");
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }

    pub async fn get_novel(
        &self,
        book_id: &str,
        on_message: Option<&impl MessageCallback>,
    ) -> Result<String> {
        let url = self.base_url.join(&format!("/novel/{}.html", book_id))?;

        self.get_html(url.as_str(), on_message, 0).await
    }

    pub async fn get_volume(
        &self,
        book_id: &str,
        volume_id: &str,
        on_message: Option<&impl MessageCallback>,
    ) -> Result<String> {
        let url = self
            .base_url
            .join(&format!("/novel/{}/vol_{}.html", book_id, volume_id))?;

        self.get_html(url.as_str(), on_message, 0).await
    }

    pub async fn get_catalog(
        &self,
        book_id: &str,
        on_message: Option<&impl MessageCallback>,
    ) -> Result<String> {
        let url = self.base_url.join(&format!("/novel/{}/catalog", book_id))?;

        self.get_html(url.as_str(), on_message, 0).await
    }

    pub async fn get_img_bytes(
        &self,
        url: &str,
        on_message: Option<&impl MessageCallback>,
    ) -> Result<Vec<u8>> {
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
        let response = client.send().await?;
        let data = response.bytes().await?;

        match utils::img_to_jpg(data.to_vec()) {
            Ok(data) => Ok(data),
            Err(err) => {
                if self.debug {
                    send_msg(on_message, String::from_utf8(data.to_vec())?.as_str());
                }
                Err(err)
            }
        }
    }
}
