use crate::epub_builder::{EpubBuilder, MetaData};
use crate::model::{BookInfo, Message, VolumeInfo};
use crate::secret::decode_text;
use crate::utils::{escape_epub_text, remove_invalid_chars, t2s};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::io::{self, Write};
use std::path::PathBuf;
use std::path::{self, absolute};
use std::thread::sleep;
// use reqwest::cookie::Jar;
use crate::client::*;
use crate::utils;
use reqwest::blocking::Client;
use reqwest::header::{HeaderValue, ACCEPT};
use scraper::{Html, Selector};

const ERROR_IMG: [&str; 8] = [
    "https://www.xlcx996.xyz/image/novel/sister01.jpg", // 3273/167199.html
    "「<img",                                           // 1744/180492.html
    "https://img3.readpai.com/3/3275/241359/263728.jpg", // 3275/241359.html
    "https://cdn-img.beixibaobao.cn/images/2vp7.png",   // 3305/168116_2.html
    "https://s6.jpg.cm/2022/07/12/Pn4pQS.jpg",          // 3342/169533_2.html
    "https://img1.imgtp.com/2022/07/26/S3ooRdwC.png",   // 3342/169525.html
    "https://img1.imgtp.com/2022/07/27/3kRju45s.png",   // 3342/169587_3.html
    "../Images/00018.jpeg",                             // 3382/249127.html
];

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
            if text.contains("對不起，該書內容已刪除") || text.contains("对不起，该书内容已删除")
            {
                message.send("该书内容已删除");
                return Err("该书内容已删除".to_string());
            }
            if text.contains("章節內容審核未通過") || text.contains("章节内容审核未通过")
            {
                message.send("该书内容审核未通过");
                return Err("该书内容审核未通过".to_string());
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
                title = Some(element.text().collect::<String>());
                break;
            }
        }
    }
    for element in document.select(&span_selector) {
        if let Some(property) = element.value().attr("class") {
            if property == "authorname" {
                author = Some(element.text().collect::<String>());
                break;
            }
        }
    }
    for element in document.select(&img_selector) {
        if let Some(property) = element.value().attr("class") {
            if property == "book-cover" {
                if let Some(property) = element.value().attr("src") {
                    cover = Some(property.to_string());
                    break;
                }
            }
        }
    }
    for element in document.select(&em_selector) {
        if let Some(property) = element.value().attr("class") {
            if property == "tag-small orange" {
                publisher = Some(element.text().collect::<String>());
            }
            if property == "tag-small red" {
                tags.push(element.text().collect::<String>());
            }
        }
    }
    for element in document.select(&content_selector) {
        description = Some(element.text().collect::<String>());
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
                let mut url_vol = None;
                let mut chapter_list: Vec<String> = Vec::new();
                let mut chapter_path_list = Vec::new();
                for element in element.select(&li_selector) {
                    if let Some(property) = element.value().attr("class") {
                        if property == "chapter-bar chapter-li" {
                            title = Some(element.text().collect::<String>());
                        }
                        if property == "volume-cover chapter-li" {
                            if let Some(element) = element.select(&a_selector).next() {
                                url_vol = Some(element.value().attr("href").unwrap().to_string());
                            }
                        }
                        if property == "chapter-li jsChapter" {
                            chapter_list.push(element.text().collect::<String>());
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
                    url_vol,
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

fn get_text(
    document: &Html,
    text: &mut Vec<Content>,
    img_list: &mut Vec<String>,
    url_base: &str,
    error_img: &HashSet<String>,
) {
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
                            if error_img.contains(&img) {
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
    pub add_catalog: bool,
    pub error_img: HashSet<String>,
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
        add_catalog: bool,
        mut error_img: HashSet<String>,
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
        error_img.extend(ERROR_IMG.iter().map(|s| s.to_string()));
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
            add_catalog,
            error_img,
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
        add_catalog: bool,
        mut error_img: HashSet<String>,
    ) -> Self {
        let client = get_client(&url_base, cookie);
        error_img.extend(ERROR_IMG.iter().map(|s| s.to_string()));
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
            add_catalog,
            error_img,
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

    fn download_single(&self, volume: &mut VolumeInfo, volume_no: usize) -> Result<(), String> {
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
        let mut text = Vec::new();
        // 章节html
        let mut text_html = Vec::new();
        // 图片url列表
        let mut img_url_list = Vec::new();
        // 图片扩展名列表
        let mut ext_list = Vec::new();
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
            let next_url = self.get_chapter_text(&url, &mut chapter_text, &mut img_url_list)?;
            for _ in img_source_list.len()..img_url_list.len() {
                img_source_list.push(url.clone());
            }
            text.push(chapter_text);
            url = next_url;
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
            img_source_list.insert(0, self.url_base.clone());
            ext_list.insert(
                0,
                String::from(".") + &self.get_ext(self.book_info.cover.clone().unwrap()),
            );
        }

        self.to_html(
            &mut text,
            &mut img_url_list,
            &mut text_html,
            &mut ext_list,
            &mut img_source_list,
        );

        // 移除空章节
        let mut remove_list = Vec::new();
        for i in 0..text_html.len() {
            if text_html[i].split("<br/>").all(|s| s.is_empty()) {
                remove_list.push(i);
            }
        }
        for i in remove_list.iter().rev() {
            text_html.remove(*i);
            volume.chapter_list.remove(*i);
        }

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
        let img_data_list = self.download_img_list(&img_url_list, &img_source_list)?;

        //制作epub
        let metadata = MetaData::new(
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
            Some(&volume.url_vol.as_ref().unwrap().replace(&self.url_base, "")),
        );
        let epub_builder = EpubBuilder::new(
            metadata,
            text_html,
            volume.chapter_list.clone(),
            img_data_list,
            ext_list,
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

    fn get_vol_desc(&self, url: &str) -> Result<Option<String>, String> {
        let url = if !url.starts_with("http") {
            format!("{}{}", self.url_base, url).as_str().to_string()
        } else {
            url.to_string()
        };
        let html = &get_html(
            &url,
            &self.client,
            &self.message,
            0,
        )?;
        let document = Html::parse_document(&html);
        let content_selector = Selector::parse("content").unwrap();
        for element in document.select(&content_selector) {
            let description = Some(element.text().collect::<String>());
            return Ok(description);
        }
        Ok(None)
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
        println!("{}", html);
        return Err("寻找章节链接失败".to_string());
    }

    fn get_save_path(&self, volume_no: &str, title: &str) -> Result<PathBuf, String> {
        let dir_name;
        let file_name;
        if self.add_number {
            dir_name = remove_invalid_chars(&format!(
                "{}",
                self.book_info.title.as_ref().unwrap(),
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
        Ok(dir.join(file_name))
    }

    fn download_img_list(
        &self,
        img_url_list: &Vec<String>,
        img_source_list: &Vec<String>,
    ) -> Result<Vec<Vec<u8>>, String> {
        self.message.send("  正在下载插图");

        let mut img_data_list = Vec::new();
        for i in 0..img_url_list.len() {
            let mut img_data = Vec::new();
            for _ in 0..50 {
                if let Ok(data) = self.download_img(&img_url_list[i]) {
                    img_data = data;
                    break;
                }
                self.message.send("\n  插图下载失败，正在重试");
                self.message.send(&format!("  {}", img_url_list[i]));
                self.message.send(&format!("  {}", img_source_list[i]));
                sleep(std::time::Duration::from_secs(5));
            }
            if img_data.is_empty() {
                return Err(format!(
                    "插图下载失败,{},{}",
                    img_url_list[i], img_source_list[i]
                ));
            }
            img_data_list.push(img_data);

            // 进度
            self.message
                .print(&format!("\r  Progress: {}/{}", i + 1, img_url_list.len())); // 使用 \r 覆盖同一行

            io::stdout().flush().unwrap(); // 强制刷新缓冲区
        }
        Ok(img_data_list)
    }

    fn download_img(&self, img_url: &str) -> Result<Vec<u8>, String> {
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
                    return Err("插图下载失败".to_string());
                }
            };
        }
        return Err("插图下载失败".to_string());
    }

    fn get_ext(&self, url: String) -> String {
        let suffixes = vec![".jpg", ".png", ".jpeg"];
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
        img_source_list: &mut Vec<String>,
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
                            img_source_list.remove(index);
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
        url: &str,
        chapter_text: &mut Vec<Content>,
        img_list: &mut Vec<String>,
    ) -> Result<String, String> {
        let html = get_html(&url, &self.client, &self.message, self.sleep_time)?;
        let document = Html::parse_document(&html);
        get_text(
            &document,
            chapter_text,
            img_list,
            &self.url_base,
            &self.error_img,
        );

        if chapter_text.is_empty() {
            self.message.send("   章节内容为空");
            return Err("Chapter text is empty".to_string());
        }

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
