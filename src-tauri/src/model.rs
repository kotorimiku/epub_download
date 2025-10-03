use serde::{self, Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct BookInfo {
    pub title: Option<String>,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub cover: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VolumeInfo {
    pub title: Option<String>,
    pub chapter_list: Vec<String>,
    pub chapter_path_list: Vec<String>,
    pub url_vol: Option<String>,
    pub volume_no: u32,
    pub cover: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Book {
    pub id: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub volume_list: Vec<Volume>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Volume {
    pub id: String,
    pub title: Option<String>,
    pub url_vol: String,
    pub volume_no: u32,
    pub updated_at: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub enum Content {
    Tag(String),
    Text(String),
    Image(String),
}

impl Content {
    pub fn is_text(&self) -> bool {
        matches!(self, Content::Text(_))
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Content::Text(text) if text.is_empty())
    }
}

#[cfg(feature = "gui")]
pub type App = tauri::AppHandle;

#[cfg(not(feature = "gui"))]
pub type App = ();
