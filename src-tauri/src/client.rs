use std::collections::HashMap;

use reqwest::{
    Client,
    header::{ACCEPT, ACCEPT_LANGUAGE, COOKIE, HeaderMap, HeaderName, HeaderValue, USER_AGENT},
};
use url::Url;

use crate::{
    bail, err,
    error::Result,
    message::send,
    model::App,
    utils::{self, t2s},
};

pub fn get_headers(
    referer: &str,
    mut cookie: &str,
    mut user_agent: &str,
    header_map: &HashMap<String, String>,
) -> Result<HeaderMap> {
    // let header_map: HashMap<String, String> = HashMap::new();
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
        cookie = "cf_clearance=nnBCOk2fKYYFXQQHuSW1aC1KqJ1rm3vt0Ixi6jzs_uc-1777606156-1.2.1.1-zgZcotszQJF_JWRQUeiOahUkcrJPiKQdy5ZtMNnv_hc_drIMUbeU26vN0.9kQ.ebhJYpFYsANmQ.u9BM490dq_23ymgNYnWVb2jxzpl4.tq8UzYWzu51i4sRAljH4l4AUvTsuJcMcfkBA9.I2C5JvYRYa5JKAemzN___CvqC0cxa9hXuorc2VHlO2yMP9jF6hAF3FQ0UecX9966SFjwYynX72olsk.ID6O3JEIEgnr7WW9sRd.BIJ3YLKkLljMpTJM36XvoNI_ZoRLec.2dLfe71D2r2NEiOXx_M4v7QAUqhsAfNKKQ75rALNPBipqctzfYgaItenPxhssp_p_10lg; jieqiVisitInfo=jieqiUserLogin%3D1778069650%2CjieqiUserId%3D220564; jieqiUserInfo=jieqiUserId%3D220564%2CjieqiUserUname%3Dkomaeda%2CjieqiUserName%3Dkomaeda%2CjieqiUserGroup%3D3%2CjieqiUserGroupName%3D%E6%99%AE%E9%80%9A%E4%BC%9A%E5%91%98%2CjieqiUserVip%3D0%2CjieqiUserHonorId%3D5%2CjieqiUserHonor%3D%E6%AF%92%E8%88%8C%2CjieqiUserToken%3D60e9be1ac557b7ade41d777be68f968c%2CjieqiCodeLogin%3D0%2CjieqiCodePost%3D0%2CjieqiUserPassword%3Dbe65b6881aa08fee8f24550b3dc712b5%2CjieqiUserLogin%3D1778069650; _ga_1K4JZ603WH=GS2.1.s1778073948$o20$g0$t1778073948$j60$l0$h0; _ga=GA1.1.518296446.1772889511; PHPSESSID=ug7bd7c2n18hn5lpa0fsun4qdk; night=1; jieqiRecentRead=2773.132372.0.1.1765563307.220564-4644.273875.0.1.1769339259.0-5027.311281.0.1.1771072425.0-1.108523.0.1.1771084732.0-3193.162550.0.1.1774450138.220564-3382.305740.0.1.1774799485.220564-2979.309218.0.1.1777606176.220564-2342.123395.0.1.1778069676.220564"
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
    headers.insert("upgrade-insecure-requests", HeaderValue::from_static("1"));
    headers.insert("sec-fetch-user", HeaderValue::from_static("?1"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    Ok(headers)
}

pub struct BiliClient {
    client: Client,
    base_url: Url,
    convert_simple_chinese: bool,
    debug: bool,
}

impl BiliClient {
    pub fn new(
        referer: &str,
        cookie: &str,
        user_agent: &str,
        header_map: &HashMap<String, String>,
        convert_simple_chinese: bool,
        debug: bool,
    ) -> Result<Self> {
        let headers = get_headers(referer, cookie, user_agent, header_map)?;
        Ok(Self {
            client: Client::builder().default_headers(headers).build()?,
            base_url: Url::parse(referer)?,
            convert_simple_chinese,
            debug,
        })
    }

    pub async fn get(&self, url: &str) -> Result<String> {
        if let Ok(res) = self.client.get(url).send().await {
            Ok(res.text().await?)
        } else {
            bail!("请求失败")
        }
    }

    pub async fn get_html(
        &self,
        url: &str,
        message: Option<&App>,
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
                    bail!("url重定向");
                }
                if let Ok(t) = res.text().await {
                    let mut text = t;
                    if self.convert_simple_chinese {
                        text = t2s(&text);
                    }
                    if text.contains("used Cloudflare to restrict access") {
                        send(message, "下载频繁，触发反爬，正在重试....");
                        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                        continue; // 重试
                    }
                    if text.contains("Just a moment...") || text.contains("403 Forbidden") {
                        send(message, "下载失败，请稍后再试");
                        bail!("下载失败，请稍后再试");
                    }
                    if text.contains("對不起，該書內容已刪除")
                        || text.contains("对不起，该书内容已删除")
                    {
                        send(message, "该书内容已删除");
                        bail!("该书内容已删除");
                    }
                    if text.contains("章節內容審核未通過") || text.contains("章节内容审核未通过")
                    {
                        send(message, "该书内容审核未通过");
                        bail!("该书内容审核未通过");
                    }
                    if text.contains("抱歉，该小说未经审核")
                        || text.contains("抱歉，該小說未經審核")
                    {
                        send(message, "该小说未经审核");
                        bail!("该小说未经审核");
                    }
                    if text.contains("抱歉，該小說不存在") || text.contains("抱歉，该小说不存在")
                    {
                        send(message, "该小说不存在");
                        bail!("该小说不存在");
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

    pub async fn get_novel(&self, book_id: &str, message: Option<&App>) -> Result<String> {
        let url = self.base_url.join(&format!("/novel/{}.html", book_id))?;

        self.get_html(url.as_str(), message, 0).await
    }

    pub async fn get_volume(
        &self,
        book_id: &str,
        volume_id: &str,
        message: Option<&App>,
    ) -> Result<String> {
        let url = self
            .base_url
            .join(&format!("/novel/{}/vol_{}.html", book_id, volume_id))?;

        self.get_html(url.as_str(), message, 0).await
    }

    pub async fn get_catalog(&self, book_id: &str, message: Option<&App>) -> Result<String> {
        let url = self.base_url.join(&format!("/novel/{}/catalog", book_id))?;

        self.get_html(url.as_str(), message, 0).await
    }

    pub async fn get_img_bytes(&self, url: &str, message: Option<&App>) -> Result<Vec<u8>> {
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
                    send(message, String::from_utf8(data.to_vec())?.as_str());
                }
                Err(err)
            }
        }
    }

    pub async fn check_update(&self) -> Result<String> {
        let url = "https://api.github.com/repos/kotorimiku/epub_download/releases/latest";
        let res = self.client.get(url).send().await?;
        let json = res.json::<serde_json::Value>().await?;
        let version = json["tag_name"]
            .as_str()
            .ok_or_else(|| err!("未获取到最新版本号"))?;
        let local_version = env!("CARGO_PKG_VERSION");
        let is_newer = utils::is_newer_version(local_version, version);
        let download_url = json["html_url"]
            .as_str()
            .ok_or_else(|| err!("未获取到下载地址"))?;
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
        let client = BiliClient::new(
            "https://www.bilinovel.com",
            "",
            "",
            &HashMap::new(),
            false,
            false,
        );
        let result = client
            .unwrap()
            .get("https://www.bilinovel.com/novel/115/catalog")
            .await
            .unwrap();
        println!("{}", result);
    }
}
