use crate::paragraph_restorer::ParagraphRestorer;
use crate::{client::*, paragraph_restorer};
use crate::epub_builder::{Body, ContentBlock, EpubBuilder, Metadata};
use crate::model::{BookInfo, Content, Message, VolumeInfo};
use crate::parse::{parse_metadata, parse_novel_text, parse_vol_desc, parse_volume_list};
use crate::secret::decode_text;
use crate::utils::remove_invalid_chars;
use anyhow::{anyhow, Result};
use regex::Regex;
use std::collections::HashSet;
use std::io::{self, Write};
use std::path::PathBuf;
use std::path::{self, absolute};
use std::thread::sleep;

pub struct Downloader {
    pub base_url: String,
    pub book_id: String,
    pub client: BiliClient,
    pub book_info: BookInfo,
    pub volume_infos: Vec<VolumeInfo>,
    pub output: String,
    pub template: String,
    pub message: Message,
    pub sleep_time: u32,
    pub add_catalog: bool,
    pub error_img: HashSet<String>,
}

fn get_metadata(book_id: &str, client: &BiliClient, message: &Message) -> Result<BookInfo> {
    Ok(parse_metadata(&client.get_novel(book_id, message)?))
}

fn get_volume_list(
    book_id: &str,
    client: &BiliClient,
    message: &Message,
) -> Result<Vec<VolumeInfo>> {
    Ok(parse_volume_list(
        &client.get_volume_catalog(book_id, message)?,
    ))
}

impl Downloader {
    pub fn new(
        base_url: String,
        book_id: String,
        output: String,
        template: String,
        message: Message,
        sleep_time: u32,
        cookie: &str,
        add_catalog: bool,
        error_img: HashSet<String>,
    ) -> Result<Self> {
        let client = BiliClient::new(&base_url, cookie);
        let book_info = get_metadata(&book_id, &client, &message)?;
        if book_info.title.is_none() {
            return Err(anyhow!("Book not found"));
        }
        let volume_infos = get_volume_list(book_id.as_str(), &client, &message)?;
        Ok(Self {
            base_url,
            book_id,
            client,
            book_info,
            volume_infos,
            output,
            template,
            message,
            sleep_time,
            add_catalog,
            error_img,
        })
    }

    pub fn new_from(
        base_url: String,
        book_id: String,
        output: String,
        book_info: BookInfo,
        volume_infos: Vec<VolumeInfo>,
        template: String,
        message: Message,
        sleep_time: u32,
        cookie: &str,
        add_catalog: bool,
        error_img: HashSet<String>,
    ) -> Self {
        let client = BiliClient::new(&base_url, cookie);
        Self {
            base_url,
            book_id,
            client,
            book_info,
            volume_infos,
            output,
            template,
            message,
            sleep_time,
            add_catalog,
            error_img,
        }
    }

    pub fn download<I>(&self, volume_no: I) -> Result<()>
    where
        I: Iterator<Item = u32>,
        I: IntoIterator<Item = u32>,
    {
        self.message.send(&format!(
            "开始下载{}，{}",
            &self.book_id,
            self.book_info.title.as_ref().unwrap()
        ));
        io::stdout().flush().unwrap();
        for no in volume_no {
            if let Some(volume) = self.volume_infos.get(no as usize - 1) {
                self.download_single(&mut volume.clone(), no as usize)?;
            }
        }
        Ok(())
    }

    fn download_single(&self, volume: &mut VolumeInfo, volume_no: usize) -> Result<()> {
        if volume.chapter_path_list.is_empty() {
            self.message.send("章节列表为空");
            return Ok(());
        }

        self.message.send(&format!(
            " -正在下载第{}卷，{}",
            volume_no,
            volume.title.as_ref().unwrap()
        ));
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

        let vol_desc = self.get_vol_desc(volume.url_vol.as_ref().unwrap())?;

        let mut url = self.get_start_next_url(volume, volume_no)?;
        // let first_url = url.clone();

        for i in 0..volume.chapter_list.len() {
            self.message.send(&format!(
                "  -正在下载第{}章，{}",
                i + 1,
                volume.chapter_list[i]
            ));
            let mut chapter_text = Vec::new();
            let next_url = self.get_chapter_text(&url, &mut chapter_text, &mut image_urls)?;
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
                .partition(|content| matches!(content, Content::Text(_)));
            // 分离封面
            if images.is_empty() {
                self.message.send("  插图页无插图，删除插图页");
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
                .filter(|content| !matches!(content, Content::Text(ref s) if s == ""))
                .collect::<Vec<_>>();
            if filter.len() > 0 {
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
        for i in 0..chapters.len() {
            if chapters[i]
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
            self.message.send("图片数量与扩展名数量不匹配");
            self.message
                .send(&format!("图片数量: {}", image_urls.len()));
            self.message
                .send(&format!("扩展名数量: {}", image_exts.len()));
            self.message.send(&format!("{:?}", image_urls));
            Err(anyhow!("图片数量与扩展名数量不匹配"))?;
        }

        //下载插图
        let img_data_list = self.download_img_list(&image_urls, &img_source_list)?;

        //制作epub
        let metadata = Metadata::new(
            &format!(
                "{}-{}",
                self.book_info.title.clone().unwrap(),
                volume.title.clone().unwrap()
            ),
            self.book_info.author.as_deref(),
            self.book_info.publisher.as_deref(),
            vol_desc.as_deref(),
            self.book_info.title.as_deref(),
            self.book_info.tags.clone(),
            Some("zh-CN"),
            Some(volume_no),
            Some(&volume.url_vol.as_ref().unwrap().replace(&self.base_url, "")),
        );
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
        self.message
            .send(&format!("\n  下载完成，保存到: {}", &path.display()));
        Ok(())
    }

    fn get_vol_desc(&self, url: &str) -> Result<Option<String>> {
        let url = if !url.starts_with("http") {
            format!("{}{}", self.base_url, url).as_str().to_string()
        } else {
            url.to_string()
        };

        let html = self.client.get_html(&url, &self.message, 0)?;
        let desc = parse_vol_desc(&html);
        Ok(desc)
    }

    fn get_start_next_url(&self, volume: &VolumeInfo, volume_no: usize) -> Result<String> {
        let mut next_url = self.base_url.clone() + &volume.chapter_path_list[0].clone();
        if next_url.contains("javascript") {
            let pre_volume = &self.volume_infos[volume_no - 2];
            let pre_url_path = pre_volume.chapter_path_list.last().unwrap();
            let url = self.base_url.clone() + pre_url_path;
            next_url = self.get_next_chapter_url(&self.client.get_html(
                &url,
                &self.message,
                self.sleep_time,
            )?)?;
        }
        Ok(next_url)
    }

    fn get_next_chapter_url(&self, html: &str) -> Result<String> {
        let url = self.get_next_url(html)?;
        if url.contains("_") {
            return self.get_next_chapter_url(&self.client.get_html(
                &url,
                &self.message,
                self.sleep_time,
            )?);
        } else {
            return Ok(url);
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

        self.message.send("寻找章节链接失败");
        println!("{}", html);
        return Err(anyhow!("寻找章节链接失败"));
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
        let dir_name =
            remove_invalid_chars(&format!("{}", self.book_info.title.as_ref().unwrap(),));

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
                .replace("{{book_title}}", &self.book_info.title.as_ref().unwrap())
                .replace("{{chapter_title}}", title)
                .replace("{{volume_no}}", volume_no),
        );

        let dir = path::Path::new(&self.output).join(dir_name);
        Ok(dir.join(format!("{}.epub", file_name)))
    }

    fn download_img_list(
        &self,
        img_url_list: &Vec<String>,
        img_source_list: &Vec<String>,
    ) -> Result<Vec<Vec<u8>>> {
        self.message.send("  正在下载插图");

        let mut img_data_list = Vec::new();
        for i in 0..img_url_list.len() {
            let mut img_data = Vec::new();
            let mut error_img = false;
            for _ in 0..50 {
                if let Ok(data) = self.client.get_img_bytes(&img_url_list[i]) {
                    img_data = data;
                    break;
                }

                if self.error_img.contains(&img_url_list[i]) {
                    self.message
                        .send(&format!("\n  错误图片，跳过: {}", img_url_list[i]));
                    error_img = true;
                    break;
                }

                self.message.send("\n  插图下载失败，正在重试");
                self.message.send(&format!("  {}", img_source_list[i]));
                self.message.send(&format!("  {}", img_url_list[i]));
                sleep(std::time::Duration::from_secs(5));
            }

            if error_img {
                img_data_list.push(img_data);
                continue;
            }

            if img_data.is_empty() {
                return Err(anyhow!(format!(
                    "插图下载失败,{},{}",
                    img_url_list[i], img_source_list[i]
                )));
            }

            img_data_list.push(img_data);

            // 进度
            self.message
                .print(&format!("\r  Progress: {}/{}", i + 1, img_url_list.len())); // 使用 \r 覆盖同一行

            io::stdout().flush().unwrap(); // 强制刷新缓冲区
        }
        Ok(img_data_list)
    }

    fn get_ext(&self, url: &str) -> String {
        let suffixes = vec!["jpg", "png", "jpeg"];
        if suffixes.iter().any(|&suffix| url.ends_with(suffix)) {
            return path::Path::new(&url)
                .extension()
                .unwrap()
                .to_string_lossy()
                .to_string();
        }
        return String::from("jpg");
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
            for i in 0..chapter_raw.len() {
                match &chapter_raw[i] {
                    Content::Image(url) => {
                        let count = image_urls.iter().filter(|&x| x == url).count();
                        let index = image_urls.iter().position(|x| x == url).unwrap();
                        if count > 1 {
                            image_urls.remove(index);
                            image_sources.remove(index);
                            remove_list.push(i);
                        } else {
                            chapter.push(ContentBlock::Image(index));
                            let ext = self.get_ext(&url);

                            if url.starts_with("//") {
                                image_urls[index] = format!("https:{}", url);
                            }
                            image_exts.push(ext);
                        }
                    }
                    Content::Text(text) => chapter.push(ContentBlock::Text(text.to_owned())),
                }
            }
            chapters.push(chapter);
        }
    }

    /// 返回下一章节url
    fn get_chapter_text(
        &self,
        url: &str,
        chapter_text: &mut Vec<Content>,
        img_list: &mut Vec<String>,
    ) -> Result<String> {
        let html = self.client.get_html(&url, &self.message, self.sleep_time)?;
        let mut chapter = Vec::new();
        parse_novel_text(&html, &mut chapter, img_list, &self.base_url);

        if chapter.is_empty() {
            self.message.send("   章节内容为空");
            return Err(anyhow!("Chapter text is empty"));
        }

        let chapter_id = url.split("/").last().unwrap().split(".").next().unwrap().split("_").next().unwrap().parse::<u64>().unwrap();

        let restorer = ParagraphRestorer::new(chapter_id);
        let chapter = restorer.restore(chapter);

        chapter_text.extend(chapter);

        // 文本解密
        if html.contains(r#"font-family: "read""#) {
            for content in &mut chapter_text.iter_mut().rev() {
                if let Content::Text(text) = content {
                    if text.contains("<br") || text.is_empty() {
                        continue;
                    }
                    let new_text = decode_text(&text);
                    println!("解密前: {}", text);
                    println!("解密后: {}", new_text);
                    *text = new_text;
                    break;
                }
            }
        }

        let next_url = self.get_next_url(&html)?;
        if next_url.contains("_") {
            self.message.send("   正在下载分页");
            return self.get_chapter_text(&next_url, chapter_text, img_list);
        } else {
            return Ok(next_url);
        }
    }
}
