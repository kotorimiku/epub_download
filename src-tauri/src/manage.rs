use std::{collections::HashMap, fs::File, io::Read};

use anyhow::Result;
use quick_xml::de::from_str;
use serde::Deserialize;
use walkdir::WalkDir;
use zip::ZipArchive;

use crate::{
    client::BiliClient,
    model::{Book, Volume},
    parse::parse_last_update,
};

#[derive(Debug, Deserialize)]
#[serde(rename = "package", rename_all = "lowercase")]
pub struct Package {
    #[serde(rename = "metadata")]
    pub metadata: Metadata,
}

#[derive(Debug, Deserialize)]
pub struct Metadata {
    #[serde(rename = "title", alias = "dc:title")]
    pub title: Option<String>,
    #[serde(rename = "creator", alias = "dc:creator")]
    pub creator: Option<String>,
    #[serde(rename = "publisher", alias = "dc:publisher")]
    pub publisher: Option<String>,
    #[serde(rename = "description", alias = "dc:description")]
    pub description: Option<String>,
    #[serde(rename = "language", alias = "dc:language")]
    pub language: Option<String>,
    #[serde(rename = "identifier", alias = "dc:identifier")]
    pub identifier: Option<String>,
    #[serde(rename = "subject", alias = "dc:subject")]
    pub subjects: Vec<String>,
    #[serde(rename = "meta")]
    pub meta: Vec<Meta>,
}

#[derive(Debug, Deserialize)]
pub struct Meta {
    #[serde(rename = "@property")]
    pub property: Option<String>,
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@content")]
    pub content: Option<String>,
    #[serde(rename = "$value")]
    pub value: Option<String>,
}

pub fn build_index(path: &str) -> Result<Vec<Book>> {
    let mut books: HashMap<String, Book> = HashMap::new();
    for entry in WalkDir::new(path) {
        let entry = entry?;
        if entry.file_type().is_file() {
            || -> Result<()> {
                let file = File::open(entry.path())?;
                let mut zip = ZipArchive::new(file)?;
                let mut package = zip.by_name("OEBPS/content.opf")?;
                let mut xml_data = String::new();
                package.read_to_string(&mut xml_data)?;
                let package: Package = from_str(&xml_data)?;

                let id = if let Some(identifier) = &package.metadata.identifier {
                    identifier
                        .replace("novel/", "")
                        .trim_start_matches("/")
                        .split("/")
                        .next()
                        .ok_or(anyhow::anyhow!("identifier is required"))?
                        .to_string()
                } else {
                    return Err(anyhow::anyhow!("identifier is required"));
                };

                let volume_id = if let Some(identifier) = &package.metadata.identifier {
                    identifier
                        .split("/")
                        .last()
                        .and_then(|s| Some(s.trim_start_matches("vol_").trim_end_matches(".html")))
                        .ok_or(anyhow::anyhow!("identifier is required"))?
                        .to_string()
                } else {
                    return Err(anyhow::anyhow!("identifier is required"));
                };

                let mut updated_at = None;
                let mut series = None;
                let mut index = None;
                for meta in package.metadata.meta {
                    if meta.property == Some("dcterms:modified".to_string()) {
                        updated_at = meta.value;
                        continue;
                    }
                    if meta.name == Some("calibre:series".to_string()) {
                        series = meta.content;
                        continue;
                    }
                    if meta.name == Some("calibre:series_index".to_string()) {
                        index = meta.content.and_then(|c| c.parse::<u32>().ok());
                        continue;
                    }
                }

                let volume = Volume {
                    id: volume_id,
                    title: package.metadata.title,
                    url_vol: package
                        .metadata
                        .identifier
                        .clone()
                        .ok_or(anyhow::anyhow!("identifier is required"))?,
                    volume_no: index.ok_or(anyhow::anyhow!("index is required"))?,
                    updated_at: updated_at.ok_or(anyhow::anyhow!("updated_at is required"))?,
                    path: entry.path().to_string_lossy().to_string(),
                };

                if books.contains_key(&id) {
                    books
                        .get_mut(&id)
                        .ok_or(anyhow::anyhow!("book is required"))?
                        .volume_list
                        .push(volume);
                } else {
                    books.insert(
                        id.to_string(),
                        Book {
                            id: id.to_string(),
                            title: series,
                            author: package.metadata.creator,
                            publisher: package.metadata.publisher,
                            tags: package.metadata.subjects,
                            description: package.metadata.description,
                            volume_list: vec![volume],
                        },
                    );
                }
                Ok(())
            }()
            .ok();
        }
    }

    let mut result: Vec<Book> = books
        .into_values()
        .map(|mut book| {
            book.volume_list.sort_by_key(|v| v.volume_no);
            book
        })
        .collect();
    result.sort_by_key(|b| b.id.clone());
    Ok(result)
}

pub fn create_index(path: &str, index_path: &str) -> Result<()> {
    let books = build_index(path)?;

    serde_json::to_writer_pretty(File::create(index_path)?, &books)?;

    Ok(())
}

pub fn get_books(index_path: &str) -> Result<Vec<Book>> {
    let str = std::fs::read_to_string(index_path).unwrap();
    let books = serde_json::from_str(&str)?;
    Ok(books)
}

pub async fn get_last_update_by_with_volume(
    client: &BiliClient,
    book_id: &str,
    volume_id: &str,
) -> Result<String> {
    let html = client.get_volume(book_id, volume_id, &None).await?;
    let last_update = parse_last_update(&html).ok_or(anyhow::anyhow!("last_update is required"))?;
    Ok(last_update)
}
