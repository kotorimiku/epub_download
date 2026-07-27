use regex::Regex;

use crate::{
    bail,
    client::BiliClient,
    error::Result,
    listener::MessageCallback,
    model::{BookInfo, Content, VolumeInfo},
    paragraph_restorer::ParagraphRestorer,
    parse::{parse_metadata, parse_novel_text, parse_vol_desc, parse_volume_list},
};

#[derive(Clone)]
pub struct BiliNovel<F = fn(&str)> {
    client: BiliClient,
    on_message: Option<F>,
}

impl<F: MessageCallback> BiliNovel<F> {
    pub fn new(client: BiliClient, on_message: Option<F>) -> Self {
        Self { client, on_message }
    }

    fn send_msg(&self, msg: &str) {
        if let Some(ref cb) = self.on_message {
            cb(msg);
        }
        println!("{}", msg);
    }

    pub async fn get_book_info(&self, book_id: &str) -> Result<BookInfo> {
        let html = self
            .client
            .get_novel(book_id, self.on_message.as_ref())
            .await?;
        Ok(parse_metadata(&html))
    }

    pub async fn get_vol_desc(&self, url: &str) -> Result<Option<String>> {
        let html = self
            .client
            .get_html(url, self.on_message.as_ref(), 0)
            .await?;
        let desc = parse_vol_desc(&html);
        Ok(desc)
    }

    pub async fn get_volume_list(&self, book_id: &str) -> Result<Vec<VolumeInfo>> {
        let catalog_html = self
            .client
            .get_catalog(book_id, self.on_message.as_ref())
            .await?;
        Ok(parse_volume_list(&catalog_html))
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
                        .get_html(url, self.on_message.as_ref(), sleep_time)
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
                    .get_html(&url, self.on_message.as_ref(), sleep_time)
                    .await?;
            } else {
                return Ok(url);
            }
        }
    }

    pub fn get_next_url(&self, html: &str) -> Result<String> {
        let re = Regex::new(r"url_next:'(.+?)'").unwrap();
        if let Some(captures) = re.captures(html) {
            if let Some(url) = captures.get(1) {
                return Ok(url.as_str().to_string());
            }
        }

        self.send_msg("寻找章节链接失败");
        println!("{}", html);
        bail!("寻找章节链接失败")
    }

    pub fn paragraph_restorer(
        &self,
        html: &str,
        img_list: &mut Vec<String>,
        url: &str,
        html_restore_callback: Option<impl Fn(&str) -> Result<String>>,
    ) -> Result<Vec<Content>> {
        let mut chapter = Vec::new();

        if let Some(callback) = html_restore_callback {
            let restored_html = callback(html)?;
            parse_novel_text(&restored_html, &mut chapter, img_list);
            return Ok(chapter);
        }

        parse_novel_text(html, &mut chapter, img_list);

        if chapter.is_empty() {
            self.send_msg("   章节内容为空");
            println!("{}", html);
            bail!("章节内容为空");
        }

        if Self::get_chapterlog_version(html)? != ParagraphRestorer::get_version() {
            bail!("章节日志版本不匹配，无法恢复章节顺序");
        }
        let chapter_id = url
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
        Ok(restorer.restore(chapter))
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
