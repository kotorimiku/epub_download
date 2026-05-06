use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    io::{self, Write},
    path::{self, PathBuf, absolute},
    thread::sleep,
};

use regex::Regex;

use crate::{
    bail,
    client::*,
    epub_builder::{Body, ContentBlock, EpubBuilder, Metadata, MetadataConfig},
    error::Result,
    message::{self, print, send},
    model::{App, BookInfo, Content, VolumeInfo},
    parse::{parse_metadata, parse_novel_text, parse_vol_desc, parse_volume_list},
    runtime::{RUN_MODE, RunMode},
    secret::decode_text,
    utils::remove_invalid_chars,
};

pub struct DownloaderConfig {
    pub base_url: String,
    pub book_id: String,
    pub output: String,
    pub template: String,
    pub sleep_time: u32,
    pub convert_simple_chinese: bool,
    pub cookie: String,
    pub user_agent: String,
    pub header_map: HashMap<String, String>,
    pub add_catalog: bool,
    pub error_img: HashSet<String>,
    pub app_handle: Option<App>,
    pub debug: bool,
}

pub struct Downloader {
    pub base_url: String,
    pub book_id: String,
    pub client: BiliClient,
    pub book_info: BookInfo,
    pub volume_infos: Vec<VolumeInfo>,
    pub output: String,
    pub template: String,
    pub sleep_time: u32,
    pub add_catalog: bool,
    pub error_img: HashSet<String>,
    pub app_handle: Option<App>,
    pub debug: bool,
}

async fn get_metadata(
    book_id: &str,
    client: &BiliClient,
    app_handle: Option<&App>,
) -> Result<BookInfo> {
    Ok(parse_metadata(
        &client.get_novel(book_id, app_handle).await?,
    ))
}

async fn get_volume_list(
    book_id: &str,
    client: &BiliClient,
    app_handle: Option<&App>,
) -> Result<Vec<VolumeInfo>> {
    Ok(parse_volume_list(
        &client.get_catalog(book_id, app_handle).await?,
    ))
}

impl Downloader {
    pub async fn new(config: DownloaderConfig) -> Result<Self> {
        let client = BiliClient::new(
            &config.base_url,
            &config.cookie,
            &config.user_agent,
            &config.header_map,
            config.convert_simple_chinese,
            config.debug,
        )?;
        let book_info = get_metadata(&config.book_id, &client, config.app_handle.as_ref()).await?;
        if book_info.title.is_none() {
            bail!("Book not found");
        }
        let volume_infos =
            get_volume_list(config.book_id.as_str(), &client, config.app_handle.as_ref()).await?;
        Ok(Self {
            base_url: config.base_url,
            book_id: config.book_id,
            client,
            book_info,
            volume_infos,
            output: config.output,
            template: config.template,
            sleep_time: config.sleep_time,
            add_catalog: config.add_catalog,
            error_img: config.error_img,
            app_handle: config.app_handle,
            debug: config.debug,
        })
    }

    pub fn new_from(
        config: DownloaderConfig,
        book_info: BookInfo,
        volume_infos: Vec<VolumeInfo>,
    ) -> Result<Self> {
        let client = BiliClient::new(
            &config.base_url,
            &config.cookie,
            &config.user_agent,
            &config.header_map,
            config.convert_simple_chinese,
            config.debug,
        )?;
        Ok(Self {
            base_url: config.base_url,
            book_id: config.book_id,
            client,
            book_info,
            volume_infos,
            output: config.output,
            template: config.template,
            sleep_time: config.sleep_time,
            add_catalog: config.add_catalog,
            error_img: config.error_img,
            app_handle: config.app_handle,
            debug: config.debug,
        })
    }

    pub async fn download<I>(&self, volume_no: I) -> Result<()>
    where
        I: Iterator<Item = u32>,
        I: IntoIterator<Item = u32>,
    {
        send(
            self.app_handle.as_ref(),
            &format!(
                "开始下载{}，{}",
                &self.book_id,
                self.book_info.title.as_ref().unwrap()
            ),
        );
        io::stdout().flush().unwrap();
        for no in volume_no {
            if let Some(volume) = self.volume_infos.get(no as usize - 1) {
                match self.download_single(&mut volume.clone(), no as usize).await {
                    Ok(()) => (),
                    Err(err) => message::send(
                        self.app_handle.as_ref(),
                        &format!("下载第{}卷失败: {:?}", no, err),
                    ),
                }
            }
        }
        Ok(())
    }

    async fn download_single(&self, volume: &mut VolumeInfo, volume_no: usize) -> Result<()> {
        if volume.chapter_path_list.is_empty() {
            send(self.app_handle.as_ref(), "章节列表为空");
            return Ok(());
        }

        send(
            self.app_handle.as_ref(),
            &format!(
                " -正在下载第{}卷，{}",
                volume_no,
                volume.title.as_ref().unwrap()
            ),
        );
        // 章节内容
        let mut chapters_raw = Vec::new();
        // 章节html
        let mut chapters = Vec::new();
        // 图片url列表
        let mut image_urls = Vec::new();
        // 图片扩展名列表
        let mut image_exts = Vec::new();
        // 图片来源列表
        let mut img_source_list = Vec::new();

        let vol_desc = self.get_vol_desc(volume.url_vol.as_ref().unwrap()).await?;

        let mut url = self.get_start_next_url(volume, volume_no).await?;
        // let first_url = url.clone();

        for i in 0..volume.chapter_list.len() {
            send(
                self.app_handle.as_ref(),
                &format!("  -正在下载第{}章，{}", i + 1, volume.chapter_list[i]),
            );
            let mut chapter_text = Vec::new();
            let next_url = self
                .get_chapter_text(&url, &mut chapter_text, &mut image_urls)
                .await?;
            for _ in img_source_list.len()..image_urls.len() {
                img_source_list.push(url.clone());
            }
            chapters_raw.push(chapter_text);
            url = next_url;
        }

        // 添加封面
        let mut add_cover = || {
            let url = if volume.cover.is_some() {
                volume.cover.clone().unwrap()
            } else {
                self.book_info.cover.clone().unwrap_or_default()
            };
            let ext = self.get_ext(&url);

            image_urls.insert(0, url);
            img_source_list.insert(0, self.base_url.clone());
            image_exts.insert(0, ext);
        };

        if volume.chapter_list[0] == "插图" {
            volume.chapter_list[0] = "彩页".to_string();
            // 分离彩页
            let color_page = chapters_raw.remove(0);
            let (info, mut images): (Vec<_>, Vec<_>) = color_page
                .into_iter()
                .partition(|content| matches!(content, Content::Text(_) | Content::Tag(_)));
            // 分离封面
            if images.is_empty() {
                send(self.app_handle.as_ref(), "  插图页无插图，删除插图页");
                volume.chapter_list.remove(0);
                // 添加封面
                add_cover();
            } else {
                images.remove(0);
                image_exts.push(self.get_ext(&image_urls[0]));
                chapters_raw.insert(0, images);
            }
            // 添加信息页
            let filter = info
                .iter()
                .filter(|content| !matches!(content, Content::Text(s) if s.is_empty()))
                .filter(|content| !matches!(content, Content::Tag(s) if s.is_empty() || s.contains("<br")))
                .collect::<Vec<_>>();
            if !filter.is_empty() {
                chapters_raw.insert(0, info);
                volume.chapter_list.insert(0, "信息".to_string());
            }
        } else {
            // 添加封面
            add_cover();
        }

        self.get_chapters(
            &mut chapters_raw,
            &mut image_urls,
            &mut chapters,
            &mut image_exts,
            &mut img_source_list,
        );

        // 移除空章节
        let mut remove_list = Vec::new();
        for (i, chapter) in chapters.iter().enumerate() {
            if chapter
                .iter()
                .all(|cb| matches!(cb, ContentBlock::Text(s) if s.is_empty()))
            {
                remove_list.push(i);
            }
        }
        for i in remove_list.iter().rev() {
            chapters.remove(*i);
            volume.chapter_list.remove(*i);
        }

        if image_urls.len() != image_exts.len() {
            send(self.app_handle.as_ref(), "图片数量与扩展名数量不匹配");
            send(
                self.app_handle.as_ref(),
                &format!("图片数量: {}", image_urls.len()),
            );
            send(
                self.app_handle.as_ref(),
                &format!("扩展名数量: {}", image_exts.len()),
            );
            send(self.app_handle.as_ref(), &format!("{:?}", image_urls));
            bail!("图片数量与扩展名数量不匹配");
        }

        //下载插图
        let img_data_list = self
            .download_img_list(&image_urls, &img_source_list)
            .await?;

        //制作epub
        let title = format!(
            "{}-{}",
            self.book_info.title.as_ref().unwrap(),
            volume.title.as_ref().unwrap()
        );
        let identifier = volume.url_vol.as_ref().unwrap().replace(&self.base_url, "");
        let metadata_config = MetadataConfig {
            title: &title,
            creator: self.book_info.author.as_deref(),
            publisher: self.book_info.publisher.as_deref(),
            description: vol_desc.as_deref(),
            series: self.book_info.title.as_deref(),
            subject: &self.book_info.tags,
            language: Some("zh-CN"),
            index: Some(volume_no),
            identifier: Some(&identifier),
        };
        let metadata: Metadata = metadata_config.into();
        let epub_builder = EpubBuilder::new(
            metadata,
            Body::Blocks(chapters),
            volume.chapter_list.clone(),
            img_data_list,
            image_exts,
            image_urls,
            self.add_catalog,
        );

        //保存文件
        let path =
            absolute(self.get_save_path(&volume_no.to_string(), volume.title.as_ref().unwrap())?)
                .unwrap();
        epub_builder.save_file(path.as_path())?;
        send(
            self.app_handle.as_ref(),
            &format!("\n  下载完成，保存到: {}", &path.display()),
        );
        Ok(())
    }

    async fn get_vol_desc(&self, url: &str) -> Result<Option<String>> {
        let url = if !url.starts_with("http") {
            format!("{}{}", self.base_url, url).as_str().to_string()
        } else {
            url.to_string()
        };

        let html = self
            .client
            .get_html(&url, self.app_handle.as_ref(), 0)
            .await?;
        let desc = parse_vol_desc(&html);
        Ok(desc)
    }

    async fn get_start_next_url(&self, volume: &VolumeInfo, volume_no: usize) -> Result<String> {
        let mut next_url = self.base_url.clone() + &volume.chapter_path_list[0].clone();
        if next_url.contains("javascript") {
            let pre_volume = &self.volume_infos[volume_no - 2];
            let pre_url_path = pre_volume.chapter_path_list.last().unwrap();
            let url = self.base_url.clone() + pre_url_path;
            next_url = self
                .get_next_chapter_url(
                    &self
                        .client
                        .get_html(&url, self.app_handle.as_ref(), self.sleep_time)
                        .await?,
                )
                .await?;
        }
        Ok(next_url)
    }

    async fn get_next_chapter_url(&self, html: &str) -> Result<String> {
        let mut current_html = html.to_string();
        loop {
            let url = self.get_next_url(&current_html)?;
            if url.contains("_") {
                current_html = self
                    .client
                    .get_html(&url, self.app_handle.as_ref(), self.sleep_time)
                    .await?;
            } else {
                return Ok(url);
            }
        }
    }

    fn get_next_url(&self, html: &str) -> Result<String> {
        let re = Regex::new(r"url_next:'(.+?)'").unwrap();
        // 使用正则表达式进行匹配
        if let Some(captures) = re.captures(html) {
            // 提取匹配到的第一个分组（即 URL）
            if let Some(url) = captures.get(1) {
                return Ok(self.base_url.clone() + url.as_str());
            }
        }

        send(self.app_handle.as_ref(), "寻找章节链接失败");
        println!("{}", html);
        bail!("寻找章节链接失败")
    }

    fn get_save_path(&self, volume_no: &str, title: &str) -> Result<PathBuf> {
        let mut template = self.template.as_str();
        template = if template == "0" {
            "{{book_title}}-{{chapter_title}}"
        } else if template == "1" {
            "{{book_title}}-[{{chapter_number}}]{{chapter_title}}"
        } else if template == "2" {
            "[{{chapter_number}}]{{chapter_title}}"
        } else if template == "3" {
            "[{{chapter_number:2}}]{{chapter_title}}"
        } else {
            template
        };
        let dir_name = remove_invalid_chars(&self.book_info.title.as_ref().unwrap().to_string());

        let re = Regex::new(r"\{\{chapter_number:(\d+)\}\}").unwrap();
        // 提取数字，格式化章节号
        let result = re
            .replace_all(template, |caps: &regex::Captures| {
                let width: usize = caps[1].parse().unwrap_or(1);
                format!("{:0>width$}", volume_no, width = width)
            })
            .to_string();

        let file_name = remove_invalid_chars(
            &result
                .replace("{{book_title}}", self.book_info.title.as_ref().unwrap())
                .replace("{{chapter_title}}", title)
                .replace("{{volume_no}}", volume_no),
        );

        let dir = path::Path::new(&self.output).join(dir_name);
        Ok(dir.join(format!("{}.epub", file_name)))
    }

    async fn download_img_list(
        &self,
        img_url_list: &[String],
        img_source_list: &[String],
    ) -> Result<Vec<Vec<u8>>> {
        send(self.app_handle.as_ref(), "  正在下载插图");

        let mut img_data_list = Vec::new();
        for i in 0..img_url_list.len() {
            let mut img_data = Vec::new();
            let mut error_img = false;
            for _ in 0..50 {
                match self
                    .client
                    .get_img_bytes(&img_url_list[i], self.app_handle.as_ref())
                    .await
                {
                    Ok(data) => {
                        img_data = data;
                        break;
                    }
                    Err(err) => {
                        message::send(
                            self.app_handle.as_ref(),
                            &format!("  插图下载失败: {:?}", err),
                        );
                    }
                };

                if self.error_img.contains(&img_url_list[i]) {
                    send(
                        self.app_handle.as_ref(),
                        &format!("\n  错误图片，跳过: {}", img_url_list[i]),
                    );
                    error_img = true;
                    break;
                }

                send(self.app_handle.as_ref(), "\n  插图下载失败，正在重试");
                send(
                    self.app_handle.as_ref(),
                    &format!("  {}", img_source_list[i]),
                );
                send(self.app_handle.as_ref(), &format!("  {}", img_url_list[i]));
                sleep(std::time::Duration::from_secs(5));
            }

            if error_img {
                // 使用一张空白图片占位，避免epub制作失败
                img_data_list.push(img_data);
                continue;
            }

            if img_data.is_empty() {
                bail!("插图下载失败,{},{}", img_url_list[i], img_source_list[i]);
            }

            img_data_list.push(img_data);

            // 进度
            print(
                self.app_handle.as_ref(),
                &format!("\r  Progress: {}/{}", i + 1, img_url_list.len()),
            ); // 使用 \r 覆盖同一行

            io::stdout().flush().unwrap(); // 强制刷新缓冲区
        }
        Ok(img_data_list)
    }

    fn get_ext(&self, url: &str) -> String {
        let suffixes = ["jpg", "png", "jpeg"];
        if suffixes.iter().any(|&suffix| url.ends_with(suffix)) {
            return path::Path::new(&url)
                .extension()
                .unwrap()
                .to_string_lossy()
                .to_string();
        }
        String::from("jpg")
    }

    fn get_chapters(
        &self,
        chapters_raw: &mut Vec<Vec<Content>>,
        image_urls: &mut Vec<String>,
        chapters: &mut Vec<Vec<ContentBlock>>,
        image_exts: &mut Vec<String>,
        image_sources: &mut Vec<String>,
    ) {
        for chapter_raw in chapters_raw {
            let mut chapter = Vec::new();
            let mut remove_list = Vec::new();
            for (i, content) in chapter_raw.iter().enumerate() {
                match content {
                    Content::Image(url) => {
                        let count = image_urls.iter().filter(|&x| x == url).count();
                        let index = image_urls.iter().position(|x| x == url).unwrap();
                        if count > 1 {
                            image_urls.remove(index);
                            image_sources.remove(index);
                            remove_list.push(i);
                        } else {
                            chapter.push(ContentBlock::Image(index));
                            let ext = self.get_ext(url);

                            if url.starts_with("//") {
                                image_urls[index] = format!("https:{}", url);
                            }
                            image_exts.push(ext);
                        }
                    }
                    Content::Text(text) => chapter.push(ContentBlock::Text(text.to_owned())),
                    Content::Tag(tag) => chapter.push(ContentBlock::Tag(tag.to_owned())),
                }
            }
            chapters.push(chapter);
        }
    }

    /// 返回下一章节url
    async fn get_chapter_text(
        &self,
        url: &str,
        chapter_text: &mut Vec<Content>,
        img_list: &mut Vec<String>,
    ) -> Result<String> {
        let html = self
            .client
            .get_html(url, self.app_handle.as_ref(), self.sleep_time)
            .await?;

        let chapter = self.paragraph_restorer(&html, img_list, url)?;

        chapter_text.extend(chapter);

        // 文本解密
        if html.contains(r#"font-family: "read""#) {
            for content in &mut chapter_text.iter_mut().rev() {
                if let Content::Text(text) = content {
                    if text.contains("<br") || text.is_empty() {
                        continue;
                    }
                    let new_text = decode_text(text);
                    println!("解密前: {}", text);
                    println!("解密后: {}", new_text);
                    *text = new_text;
                    break;
                }
                if let Content::Tag(tag) = content {
                    if tag.contains("<br") || tag.is_empty() {
                        continue;
                    }
                    let new_tag = decode_text(tag);
                    println!("解密前: {}", tag);
                    println!("解密后: {}", new_tag);
                    *tag = new_tag;
                    break;
                }
            }
        }

        let mut current_url = self.get_next_url(&html)?;
        while current_url.contains("_") {
            send(self.app_handle.as_ref(), "   正在下载分页");
            let html = self
                .client
                .get_html(&current_url, self.app_handle.as_ref(), self.sleep_time)
                .await?;

            let chapter = self.paragraph_restorer(&html, img_list, &current_url)?;

            chapter_text.extend(chapter);

            current_url = self.get_next_url(&html)?;
        }
        Ok(current_url)
    }

    fn paragraph_restorer(
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
        parse_novel_text(html.as_ref(), &mut chapter, img_list, &self.base_url);

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
        let version = Downloader::get_chapterlog_version(&html).unwrap();
        println!("version: {}", version);
    }
}
