use std::borrow::Cow;

use regex::Regex;

use crate::{
    bail,
    client::BiliClient,
    error::Result,
    message::send,
    model::{App, BookInfo, Content, VolumeInfo},
    parse::{parse_metadata, parse_novel_text, parse_vol_desc, parse_volume_list},
    runtime::{RUN_MODE, RunMode},
};

pub struct BiliNovel {
    client: BiliClient,
    app_handle: Option<App>,
    debug: bool,
}

impl BiliNovel {
    pub fn new(client: BiliClient, app_handle: Option<App>, debug: bool) -> Self {
        Self {
            client,
            app_handle,
            debug,
        }
    }

    pub async fn get_book_info(&self, book_id: &str) -> Result<BookInfo> {
        let html = self.client.get_novel(book_id, None).await?;
        Ok(parse_metadata(&html))
    }

    pub async fn get_vol_desc(&self, url: &str) -> Result<Option<String>> {
        let html = self
            .client
            .get_html(&url, self.app_handle.as_ref(), 0)
            .await?;
        let desc = parse_vol_desc(&html);
        Ok(desc)
    }

    pub async fn get_volume_list(&self, book_id: &str) -> Result<Vec<VolumeInfo>> {
        Ok(parse_volume_list(
            &self
                .client
                .get_catalog(book_id, self.app_handle.as_ref())
                .await?,
        ))
    }

    pub async fn get_start_next_url<'a>(
        &self,
        volume: &VolumeInfo,
        _volume_no: usize,
        pre_volume: impl Fn() -> &'a VolumeInfo,
        sleep_time: u32,
    ) -> Result<String> {
        let mut next_url = volume.chapter_path_list[0].clone();
        if next_url.contains("javascript") {
            let pre_volume = pre_volume();
            let pre_url_path = pre_volume.chapter_path_list.last().unwrap();
            let url = pre_url_path;
            next_url = self
                .get_next_chapter_url(
                    &self
                        .client
                        .get_html(&url, self.app_handle.as_ref(), sleep_time)
                        .await?,
                    sleep_time,
                )
                .await?;
        }
        Ok(next_url)
    }

    async fn get_next_chapter_url(&self, html: &str, sleep_time: u32) -> Result<String> {
        let mut current_html = html.to_string();
        loop {
            let url = self.get_next_url(&current_html)?;
            if url.contains("_") {
                current_html = self
                    .client
                    .get_html(&url, self.app_handle.as_ref(), sleep_time)
                    .await?;
            } else {
                return Ok(url);
            }
        }
    }

    pub fn get_next_url(&self, html: &str) -> Result<String> {
        let re = Regex::new(r"url_next:'(.+?)'").unwrap();
        // 使用正则表达式进行匹配
        if let Some(captures) = re.captures(html) {
            // 提取匹配到的第一个分组（即 URL）
            if let Some(url) = captures.get(1) {
                return Ok(url.as_str().to_string());
            }
        }

        send(self.app_handle.as_ref(), "寻找章节链接失败");
        println!("{}", html);
        bail!("寻找章节链接失败")
    }

    pub fn paragraph_restorer(
        &self,
        html: &str,
        img_list: &mut Vec<String>,
        _url: &str,
    ) -> Result<Vec<Content>> {
        // #[cfg(feature = "gui")]
        // let html = &crate::event::html(self.app_handle.as_ref().unwrap(), html)?;

        let html = match *RUN_MODE.lock() {
            RunMode::Gui => {
                #[cfg(feature = "gui")]
                {
                    use std::borrow::Cow;

                    match crate::event::html(self.app_handle.as_ref().unwrap(), html) {
                        Ok(html) => Cow::Owned(html),
                        Err(err) => {
                            if self.debug {
                                send(self.app_handle.as_ref(), html);
                            }
                            bail!("章节内容解析失败: {:?}", err);
                        }
                    }
                }

                #[cfg(not(feature = "gui"))]
                bail!("当前构建未启用 gui feature");
            }
            RunMode::Cli => Cow::Borrowed(html),
        };

        let mut chapter = Vec::new();
        parse_novel_text(html.as_ref(), &mut chapter, img_list);

        if chapter.is_empty() {
            send(self.app_handle.as_ref(), "   章节内容为空");
            println!("{}", html);
            bail!("章节内容为空");
        }

        let chapter = match *RUN_MODE.lock() {
            RunMode::Gui => chapter,
            RunMode::Cli => {
                use crate::paragraph_restorer::ParagraphRestorer;
                if Self::get_chapterlog_version(html.as_ref())? != ParagraphRestorer::get_version()
                {
                    bail!("章节日志版本不匹配，无法恢复章节顺序");
                }
                let chapter_id = _url
                    .split("/")
                    .last()
                    .unwrap()
                    .split(".")
                    .next()
                    .unwrap()
                    .split("_")
                    .next()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap();

                let restorer = ParagraphRestorer::new(chapter_id);
                restorer.restore(chapter)
            }
        };

        Ok(chapter)
    }

    fn get_chapterlog_version(html: &str) -> Result<String> {
        let re = Regex::new(r"chapterlog\.js\?v([\w.]+)").unwrap();
        if let Some(captures) = re.captures(html)
            && let Some(version) = captures.get(0)
        {
            return Ok(version.as_str().to_string());
        }

        bail!("chapterlog.js version not found")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[ignore]
    #[tokio::test]
    async fn test_get_chapterlog_version() {
        let config = Config::default();
        let client = BiliClient::new(
            config.base_url.as_str(),
            config.cookie.as_str(),
            config.user_agent.as_str(),
            &config.headers,
            config.convert_simple_chinese,
            config.debug,
        )
        .unwrap();
        let html = client
            .get("https://www.bilinovel.com/novel/1/108523.html")
            .await
            .unwrap();
        let version = BiliNovel::get_chapterlog_version(&html).unwrap();
        println!("version: {}", version);
    }
}
