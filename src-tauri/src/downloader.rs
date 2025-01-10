use crate::epub_builder::{EpubBuilder, MetaData};
use crate::model::{BookInfo, Message, VolumeInfo};
use crate::secret::{decode_text, get_secret_map};
use crate::utils::{remove_invalid_chars, t2s, escape_epub_text};
use regex::Regex;
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::path::{self, absolute};
use std::thread::sleep;
// use reqwest::cookie::Jar;
use crate::utils;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, COOKIE, USER_AGENT};
use scraper::{Html, Selector};
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;

fn get_headers(referer: &str, cookie: &str) -> HeaderMap {
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

pub fn get_client(referer: &str, cookie: &str) -> Client {
    // 创建一个 Cookie Jar
    let headers = get_headers(referer, cookie);
    // let jar = Arc::new(Jar::default());
    Client::builder().default_headers(headers).build().unwrap()
}

fn get_html(
    url: &str,
    client: &Client,
    message: &Message,
    sleep_time: u64,
) -> Result<String, String> {
    println!("{url}");

    std::thread::sleep(std::time::Duration::from_secs(sleep_time));
    if let Ok(res) = client.get(url).send() {
        if let Ok(t) = res.text() {
            let mut text = t;
            if url.contains("tw.linovelib.com") {
                text = t2s(&text);
            }
            if text.contains("used Cloudflare to restrict access") {
                message.send("下载频繁，触发反爬，正在重试....");
                std::thread::sleep(std::time::Duration::from_secs(3));
                return get_html(url, client, message, sleep_time);
            }
            if text.contains("Just a moment...") || text.contains("403 Forbidden") {
                message.send("下载失败，请稍后再试");
                return Err("下载失败，请稍后再试".to_string());
            }
            if text.contains("對不起，該書內容已刪除") || text.contains("对不起，该书内容已删除") {
                message.send("该书内容已删除");
                return Err("该书内容已删除".to_string());
            }
            if text.contains("通告～客戶端停用中")
                || text.contains("通告～客户端停用中")
                || text.contains("內容加载失败")
                || text.contains("手机版页面由于相容性问题暂不支持电脑端阅读")
            {
                println!("{}", &text);
                message.send("无法下载完整内容，正在重试....");
                std::thread::sleep(std::time::Duration::from_secs(3));
                return get_html(url, client, message, sleep_time);
            }
            // novel/2492/139388_6.html包含error code
            // if text.contains("error code") {
            //     message.send("请求失败，正在重试....");
            //     std::thread::sleep(std::time::Duration::from_secs(3));
            //     return get_html(url, client, message, sleep_time);
            // }
            return Ok(text);
        }
    }
    message.send("请求失败，正在重试....");
    std::thread::sleep(std::time::Duration::from_secs(3));
    return get_html(url, client, message, sleep_time);
}

pub fn get_meta_data(html: &str) -> BookInfo {
    let document = Html::parse_document(html);
    let h1_selector = Selector::parse("h1").unwrap();
    let span_selector = Selector::parse("span").unwrap();
    let content_selector = Selector::parse("content").unwrap();
    let em_selector = Selector::parse("em").unwrap();
    let img_selector = Selector::parse("img").unwrap();

    let mut title: Option<String> = None;
    let mut author: Option<String> = None;
    let mut publisher: Option<String> = None;
    let mut tags: Vec<String> = Vec::new();
    let mut description: Option<String> = None;
    let mut cover: Option<String> = None;

    for element in document.select(&h1_selector) {
        if let Some(property) = element.value().attr("class") {
            if property == "book-title" {
                title = Some(escape_epub_text(element.text().collect::<String>().as_str()));
                break;
            }
        }
    }
    for element in document.select(&span_selector) {
        if let Some(property) = element.value().attr("class") {
            if property == "authorname" {
                author = Some(escape_epub_text(element.text().collect::<String>().as_str()));
                break;
            }
        }
    }
    for element in document.select(&img_selector) {
        if let Some(property) = element.value().attr("class") {
            if property == "book-cover" {
                if let Some(property) = element.value().attr("src") {
                    cover = Some(escape_epub_text(property));
                    break;
                }
            }
        }
    }
    for element in document.select(&em_selector) {
        if let Some(property) = element.value().attr("class") {
            if property == "tag-small orange" {
                publisher = Some(escape_epub_text(element.text().collect::<String>().as_str()));
            }
            if property == "tag-small red" {
                tags.push(escape_epub_text(element.text().collect::<String>().as_str()));
            }
        }
    }
    for element in document.select(&content_selector) {
        description = Some(escape_epub_text(element.text().collect::<String>().as_str()));
        break;
    }
    BookInfo {
        title,
        author,
        publisher,
        tags,
        description,
        cover,
    }
}

pub fn get_volume_list(html: &str) -> Vec<VolumeInfo> {
    let document = Html::parse_document(html);
    let ul_selector = Selector::parse("ul").unwrap();
    let li_selector = Selector::parse("li").unwrap();
    let a_selector = Selector::parse("a").unwrap();
    let mut volume_list: Vec<VolumeInfo> = Vec::new();

    for element in document.select(&ul_selector) {
        if let Some(property) = element.value().attr("class") {
            if property == "volume-chapters" {
                let mut title = None;
                let mut chapter_list: Vec<String> = Vec::new();
                let mut chapter_path_list = Vec::new();
                for element in element.select(&li_selector) {
                    if let Some(property) = element.value().attr("class") {
                        if property == "chapter-bar chapter-li" {
                            title = Some(escape_epub_text(element.text().collect::<String>().as_str()));
                        }
                        if property == "chapter-li jsChapter" {
                            chapter_list.push(escape_epub_text(element.text().collect::<String>().as_str()));
                            if let Some(element) = element.select(&a_selector).next() {
                                chapter_path_list
                                    .push(element.value().attr("href").unwrap().to_string());
                            }
                        }
                    }
                }
                volume_list.push(VolumeInfo {
                    title,
                    chapter_list,
                    chapter_path_list,
                });
            }
        }
    }
    volume_list
}

#[derive(Debug)]
enum Content {
    Text(String),
    Image(String),
}

fn get_text(document: &Html, text: &mut Vec<Content>, img_list: &mut Vec<String>, url_base: &str) {
    let div_selector = Selector::parse("div").unwrap();

    for element in document.select(&div_selector) {
        if let Some(property) = element.value().attr("id") {
            if property == get_html_property_map(url_base).get("get_text_div").unwrap() {
                for child in element.child_elements() {
                    if child.value().name() == "img" {
                        let mut img = None;
                        if let Some(data_src) = child.value().attr("data-src") {
                            img = Some(data_src.to_string());
                        } else if let Some(src) = child.value().attr("src") {
                            img = Some(src.to_string());
                        }
                        if let Some(img) = img {
                            // 排除1744/180492.html的错误img标签
                            if img.len() < 8 {
                                continue;
                            }
                            text.push(Content::Image(img.clone()));
                            img_list.push(img);
                        }
                    } else if child.value().name().len() > 1 && child.value().name().contains("p") {
                        continue;
                    } else if child.value().name() == "div" && child.value().attr("class").is_some()
                    {
                        continue;
                    } else {
                        let t: String = child.text().collect::<String>().trim().to_string();
                        let html = child.html();
                        if !t.contains("function")
                            && !t.contains("Note: 请不要")
                            && !t.contains("= window.")
                        {
                            if t.is_empty() {
                                text.push(Content::Text("<br/>".to_string()));
                            } else {
                                text.push(Content::Text(html.replace(&t, &escape_epub_text(&t))));
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn get_html_property_map(url_base: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if url_base == "https://tw.linovelib.com" {
        map.insert(String::from("get_text_div"), String::from("acontent"));
    } else if url_base == "https://www.bilinovel.com" {
        map.insert(String::from("get_text_div"), String::from("acontent"));
    }
    map
}

pub struct Downloader {
    pub url_base: String,
    pub book_id: String,
    pub client: Client,
    pub book_info: BookInfo,
    pub volume_list: Vec<VolumeInfo>,
    pub output_path: String,
    pub add_number: bool,
    pub message: Message,
    pub sleep_time: u64,
}

impl Downloader {
    pub fn new(
        url_base: String,
        book_id: String,
        output_path: String,
        add_number: bool,
        message: Message,
        sleep_time: u64,
        cookie: &str,
    ) -> Result<Self, String> {
        let client = get_client(&url_base, cookie);
        let book_info = get_meta_data(&get_html(
            &format!("{}/novel/{}.html", url_base, book_id),
            &client,
            &message,
            0,
        )?);
        if book_info.title.is_none() {
            return Err("Book not found".to_string());
        }
        let volume_list = get_volume_list(&get_html(
            &format!("{}/novel/{}/catalog", url_base, book_id),
            &client,
            &message,
            0,
        )?);
        Ok(Self {
            url_base,
            book_id,
            client,
            book_info,
            volume_list,
            output_path,
            add_number,
            message,
            sleep_time,
        })
    }

    pub fn new_from(
        url_base: String,
        book_id: String,
        output_path: String,
        book_info: BookInfo,
        volume_list: Vec<VolumeInfo>,
        add_number: bool,
        message: Message,
        sleep_time: u64,
        cookie: &str,
    ) -> Self {
        let client = get_client(&url_base, cookie);
        Self {
            url_base,
            book_id,
            client,
            book_info,
            volume_list,
            output_path,
            add_number,
            message,
            sleep_time,
        }
    }

    pub fn download<I>(&self, volume_no: I) -> Result<(), String>
    where
        I: Iterator<Item = usize>,
        I: IntoIterator<Item = usize>,
    {
        self.message.send(&format!(
            "开始下载{}，{}",
            &self.book_id,
            self.book_info.title.as_ref().unwrap()
        ));
        io::stdout().flush().unwrap();
        for no in volume_no {
            if let Some(volume) = self.volume_list.get(no - 1) {
                self.download_single(&mut volume.clone(), no)?;
            }
        }
        Ok(())
    }

    pub fn download_single(&self, volume: &mut VolumeInfo, volume_no: usize) -> Result<(), String> {
        if volume.chapter_path_list.is_empty() {
            self.message.send("章节列表为空");
            return Ok(());
        }

        self.message.send(&format!(
            " -正在下载第{}卷，{}",
            volume_no,
            volume.title.as_ref().unwrap()
        ));
        let mut text = Vec::new();
        let mut text_html = Vec::new();
        let mut img_url_list = Vec::new();
        let mut ext_list = Vec::new();

        let mut next_url = self.get_start_next_url(volume, volume_no)?;
        let first_url = next_url.clone();

        for i in 0..volume.chapter_list.len() {
            self.message.send(&format!(
                "  -正在下载第{}章，{}",
                i + 1,
                volume.chapter_list[i]
            ));
            let mut chapter_text = Vec::new();
            next_url = self.get_chapter_text(next_url, &mut chapter_text, &mut img_url_list)?;
            text.push(chapter_text);
        }

        if volume.chapter_list[0] == "插图" {
            volume.chapter_list[0] = "彩页".to_string();
            // 分离彩页
            let color_page = text.remove(0);
            let (texts, mut images): (Vec<_>, Vec<_>) = color_page
                .into_iter()
                .partition(|content| matches!(content, Content::Text(_)));
            // 分离封面、
            images.remove(0);
            ext_list.push(String::from(".") + &self.get_ext(img_url_list[0].clone()));
            text.insert(0, images);
            // 添加信息页
            let filter = texts
                .iter()
                .filter(|content| !matches!(content, Content::Text(ref s) if s == ""))
                .collect::<Vec<_>>();
            if filter.len() > 0 {
                text.insert(0, texts);
                volume.chapter_list.insert(0, "信息".to_string());
            }
        } else {
            img_url_list.insert(0, self.book_info.cover.clone().unwrap());
            ext_list.insert(0, String::from(".") + &self.get_ext(self.book_info.cover.clone().unwrap()));
        }

        self.to_html(&mut text, &mut img_url_list, &mut text_html, &mut ext_list);

        if img_url_list.len() != ext_list.len() {
            self.message.send("图片数量与扩展名数量不匹配");
            self.message
                .send(&format!("图片数量: {}", img_url_list.len()));
            self.message
                .send(&format!("扩展名数量: {}", ext_list.len()));
            self.message.send(&format!("{:?}", img_url_list));
            Err("图片数量与扩展名数量不匹配".to_string())?;
        }

        //下载插图
        let img_data_list = self.download_img_list(&img_url_list);

        //制作epub
        let metadata = MetaData {
            title: format!(
                "{}-{}",
                self.book_info.title.clone().unwrap(),
                volume.title.clone().unwrap()
            ),
            creator: self.book_info.author.clone(),
            publisher: self.book_info.publisher.clone(),
            description: self.book_info.description.clone(),
            series: self.book_info.title.clone(),
            subject: self.book_info.tags.clone(),
            language: Some("zh-CN".to_string()),
            index: Some(volume_no),
            identifier: Some(first_url),
        };
        let epub_builder = EpubBuilder::new(
            metadata,
            text_html,
            volume.chapter_list.clone(),
            img_data_list,
            ext_list,
        );

        //保存文件
        let path =
            absolute(self.get_save_path(&volume_no.to_string(), volume.title.as_ref().unwrap())?)
                .unwrap();
        self.save_file(epub_builder.build_epub(), &path);
        self.message
            .send(&format!("\n  下载完成，保存到: {}", &path.display()));
        Ok(())
    }

    fn get_start_next_url(&self, volume: &VolumeInfo, volume_no: usize) -> Result<String, String> {
        let mut next_url = self.url_base.clone() + &volume.chapter_path_list[0].clone();
        if next_url.contains("javascript") {
            let pre_volume = &self.volume_list[volume_no - 2];
            let pre_url_path = pre_volume.chapter_path_list.last().unwrap();
            next_url = self.get_next_chapter_url(&get_html(
                &(self.url_base.clone() + pre_url_path),
                &self.client,
                &self.message,
                self.sleep_time,
            )?)?;
        }
        Ok(next_url)
    }

    fn get_next_chapter_url(&self, html: &str) -> Result<String, String> {
        let url = self.get_next_url(html)?;
        if url.contains("_") {
            return self.get_next_chapter_url(&get_html(
                &url,
                &self.client,
                &self.message,
                self.sleep_time,
            )?);
        } else {
            return Ok(url);
        }
    }

    fn get_next_url(&self, html: &str) -> Result<String, String> {
        let re = Regex::new(r"url_next:'(.+?)'").unwrap();
        // 使用正则表达式进行匹配
        if let Some(captures) = re.captures(html) {
            // 提取匹配到的第一个分组（即 URL）
            if let Some(url) = captures.get(1) {
                return Ok(self.url_base.clone() + url.as_str());
            }
        }

        self.message.send("寻找章节链接失败");
        return Err("寻找章节链接失败".to_string());
    }

    fn get_save_path(&self, volume_no: &str, title: &str) -> Result<PathBuf, String> {
        let dir_name;
        let file_name;
        if self.add_number {
            dir_name = remove_invalid_chars(&format!(
                "{}[{}]",
                self.book_info.title.as_ref().unwrap(),
                &self.book_id
            ));
            file_name = remove_invalid_chars(&format!(
                "{}-[{}]{}.epub",
                self.book_info.title.as_ref().unwrap(),
                volume_no,
                title
            ));
        } else {
            dir_name =
                remove_invalid_chars(&format!("{}", self.book_info.title.as_ref().unwrap(),));
            file_name = remove_invalid_chars(&format!(
                "{}-{}.epub",
                self.book_info.title.as_ref().unwrap(),
                title
            ));
        }

        let dir = path::Path::new(&self.output_path).join(dir_name);
        match create_dir_all(&dir) {
            Ok(_) => (),
            Err(e) => {
                self.message.send(&format!("创建目录失败: {}", e));
                return Err(String::from("创建目录失败"));
            }
        }
        Ok(dir.join(file_name))
    }

    fn download_img_list(&self, img_url_list: &Vec<String>) -> Vec<Vec<u8>> {
        self.message.send("  正在下载插图");

        let mut img_data_list = Vec::new();
        for i in 0..img_url_list.len() {
            img_data_list.push(self.download_img(&img_url_list[i]));

            // 进度
            self.message
                .print(&format!("\r  Progress: {}/{}", i + 1, img_url_list.len())); // 使用 \r 覆盖同一行

            io::stdout().flush().unwrap(); // 强制刷新缓冲区
        }
        img_data_list
    }

    fn download_img(&self, img_url: &str) -> Vec<u8> {
        let mut client = self.client.get(img_url).header(
            ACCEPT,
            "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8",
        );
        if img_url.contains("masiro") {
            client = client.header(
                "Referer",
                HeaderValue::from_static("https://www.masiro.me/"),
            );
        }
        if let Ok(response) = client.send() {
            let length = response.content_length().unwrap_or(0);
            let code = response.status().as_u16();
            let data = response.bytes().unwrap();

            if code == 404 {
                self.message
                    .send(&format!("\n  插图下载失败，404 Not Found {}", img_url));
                return Vec::new();
            }

            if length != data.len() as u64 {
                return self.download_img(img_url);
            }

            return match utils::img_to_jpg(data.to_vec()) {
                Ok(data) => data,
                Err(_) => {
                    self.message.send("\n  插图下载失败，正在重试");
                    self.message.send(&format!("  {}", img_url));
                    sleep(std::time::Duration::from_secs(5));
                    self.download_img(img_url)
                }
            };
        }
        self.message.send("\n  插图下载失败，正在重试");
        self.message.send(&format!("  {}", img_url));
        sleep(std::time::Duration::from_secs(5));
        self.download_img(img_url)
    }

    fn save_file(&self, file_map: HashMap<String, Vec<u8>>, path: &PathBuf) {
        let zip_file = File::create(path).unwrap();
        let mut zip_writer = zip::ZipWriter::new(zip_file);
        // 设置默认的文件压缩选项
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored); // 使用存储压缩（不压缩），可以根据需要更改
        for (file_name, file_data) in file_map {
            zip_writer.start_file(file_name, options).unwrap();
            zip_writer.write_all(&file_data).unwrap();
        }
        zip_writer.finish().unwrap();
    }

    fn get_ext(&self, url: String) -> String {
        let mut url = url;
        let suffixes = vec![".jpg", ".png", ".jpeg"];
        if !url.starts_with("http") {
            url = self.url_base.clone() + &url;
        }
        if suffixes.iter().any(|&suffix| url.ends_with(suffix)) {
            return path::Path::new(&url)
                .extension()
                .unwrap()
                .to_string_lossy()
                .to_string();
        }
        return String::from("jpg");
    }

    fn to_html(
        &self,
        text: &mut Vec<Vec<Content>>,
        img_url_list: &mut Vec<String>,
        text_html: &mut Vec<String>,
        ext_list: &mut Vec<String>,
    ) {
        for chapter in text {
            let mut remove_list = Vec::new();
            for i in 0..chapter.len() {
                match &chapter[i] {
                    Content::Image(url) => {
                        let count = img_url_list.iter().filter(|&x| x == url).count();
                        let index = img_url_list.iter().position(|x| x == url).unwrap();
                        if count > 1 {
                            img_url_list.remove(index);
                            remove_list.push(i);
                        } else {
                            let ext = self.get_ext(url.clone());

                            if url.starts_with("//") {
                                img_url_list[index] = format!("https:{}", url);
                            }

                            chapter[i] = Content::Image(format!(
                                "<img src=\"../Images/{}.{}\" alt=\"{}\" />",
                                format!("{:0>3}", index),
                                &ext,
                                url
                            ));
                            ext_list.push(String::from(".") + &ext);
                        }
                    }
                    Content::Text(_) => {}
                }
            }
            for i in remove_list.iter().rev() {
                chapter.remove(*i);
            }
            text_html.push(self.to_chapter_html(&*chapter));
        }
    }

    fn to_chapter_html(&self, chapter_text: &Vec<Content>) -> String {
        return chapter_text
            .iter() // 遍历 Vec<Content> 的每个元素
            .map(|content| match content {
                Content::Text(s) => s.clone(),
                Content::Image(s) => s.clone(),
            })
            .collect::<Vec<String>>()
            .join("\n    ");
    }

    fn get_chapter_text(
        &self,
        url: String,
        chapter_text: &mut Vec<Content>,
        img_list: &mut Vec<String>,
    ) -> Result<String, String> {
        let html = get_html(&url, &self.client, &self.message, self.sleep_time)?;
        let document = Html::parse_document(&html);
        get_text(&document, chapter_text, img_list, &self.url_base);

        if chapter_text.is_empty() {
            self.message.send("   章节内容为空");
            return Err("Chapter text is empty".to_string());
        }

        if html.contains(r#"read|sheet|family"#) {
            for content in &mut chapter_text.iter_mut().rev() {
                if let Content::Text(text) = content {
                    let new_text = decode_text(&text, &get_secret_map());
                    *text = new_text;
                    break;
                }
            }
        }

        let next_url = self.get_next_url(&html)?;
        if next_url.contains("_") {
            self.message.send("   正在下载分页");
            return self.get_chapter_text(next_url.to_string(), chapter_text, img_list);
        } else {
            return Ok(next_url.to_string());
        }
    }
}
