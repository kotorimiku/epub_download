use std::collections::HashMap;

use scraper::{Html, Selector};

use crate::{
    model::{BookInfo, Content, VolumeInfo},
    utils,
};

pub fn parse_metadata(html: &str) -> BookInfo {
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

pub fn parse_volume_list(html: &str) -> Vec<VolumeInfo> {
    let document = Html::parse_document(html);
    let ul_selector = Selector::parse("ul").unwrap();
    let li_selector = Selector::parse("li").unwrap();
    let a_selector = Selector::parse("a").unwrap();
    let img_selector = Selector::parse("img").unwrap();
    let mut volume_list: Vec<VolumeInfo> = Vec::new();

    for element in document.select(&ul_selector) {
        if let Some(property) = element.value().attr("class") {
            if property == "volume-chapters" {
                let mut title = None;
                let mut url_vol = None;
                let mut chapter_list: Vec<String> = Vec::new();
                let mut chapter_path_list = Vec::new();
                let mut cover = None;
                for element in element.select(&li_selector) {
                    if let Some(property) = element.value().attr("class") {
                        if property == "chapter-bar chapter-li" {
                            title = Some(element.text().collect::<String>());
                        }
                        if property == "volume-cover chapter-li" {
                            if let Some(element) = element.select(&a_selector).next() {
                                url_vol = Some(element.value().attr("href").unwrap().to_string());
                            }
                            for element in element.select(&img_selector) {
                                if let Some(src) = element.value().attr("data-src") {
                                    cover = Some(src.to_string());
                                } else if let Some(src) = element.value().attr("src") {
                                    cover = Some(src.to_string());
                                }
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
                    volume_no: (volume_list.len() + 1).try_into().unwrap(),
                    cover,
                });
            }
        }
    }
    volume_list
}

pub fn parse_novel_text(
    html: &str,
    text: &mut Vec<Content>,
    img_list: &mut Vec<String>,
    _url_base: &str,
) {
    let document = Html::parse_document(&html);
    let div_selector = Selector::parse("div").unwrap();

    for element in document.select(&div_selector) {
        if let Some(property) = element.value().attr("id") {
            if property == "acontent" {
                for child in element.child_elements() {
                    if child.value().name() == "img" {
                        let mut img = None;
                        if let Some(data_src) = child.value().attr("data-src") {
                            img = Some(data_src.to_string());
                        } else if let Some(src) = child.value().attr("src") {
                            img = Some(src.to_string());
                        }
                        if let Some(img) = img {
                            // if error_img.contains(&img) {
                            //     continue;
                            // }
                            text.push(Content::Image(img.clone()));
                            img_list.push(img);
                        }
                    } else if child.value().name().len() > 1 && child.value().name().contains("p") {
                        continue;
                    } else if child.value().name() == "div" && child.value().attr("class").is_some()
                    {
                        continue;
                    } else {
                        let t = child.text().collect::<String>();
                        if t.contains("function")
                            || t.contains("Note: 请不要")
                            || t.contains("= window.")
                        {
                            continue;
                        }
                        let raw_text = child.text();
                        let mut texts = HashMap::new();
                        for text in raw_text {
                            texts.insert(text, utils::escape_epub_text(&text.trim()));
                        }

                        let mut html = child.html();
                        if let Some(class) = child.value().attr("class") {
                            html = html.replace(format!(" class=\"{}\"", class).as_str(), "");
                        }

                        for (raw_t, t) in texts {
                            html = html.replace(raw_t, &t);
                        }

                        if t.is_empty() {
                            html = String::from("<br/>");
                        }

                        text.push(Content::Tag(html));
                    }
                }
            }
        }
    }
}

pub fn parse_vol_desc(html: &str) -> Option<String> {
    let document = Html::parse_document(&html);
    let content_selector = Selector::parse("content").unwrap();
    for element in document.select(&content_selector) {
        let description = Some(element.text().collect::<String>());
        return description;
    }
    None
}

pub fn parse_last_update(novel_html: &str) -> Option<String> {
    let document = Html::parse_document(&novel_html);
    let a_selector = Selector::parse("a.book-meta.book-status").unwrap();
    let div_selector = Selector::parse("div.book-meta-l").unwrap();
    for element in document.select(&a_selector) {
        for element in element.select(&div_selector) {
            let mut direct_text = String::new();
            for child in element.children() {
                println!("child: {:?}", child.value());
                if child.value().is_text() {
                    direct_text.push_str(child.value().as_text().unwrap());
                }
            }
            let cleaned_text = direct_text.trim();
            if !cleaned_text.is_empty() {
                return Some(cleaned_text.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::client;

    use super::*;

    #[tokio::test]
    async fn test_parse_last_update() {
        let html = client::BiliClient::new("https://www.bilinovel.com", "", "")
            .unwrap()
            .get_novel("1", &None)
            .await
            .unwrap();
        let last_update = parse_last_update(&html);
        println!("last_update: {:?}", last_update);
    }
}
