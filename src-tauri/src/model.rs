use serde::{self, Serialize, Deserialize};
use tauri::Emitter;

#[derive(Debug, Serialize, Deserialize)]
pub struct BookInfo {
    pub title: Option<String>,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub cover: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeInfo {
    pub title: Option<String>,
    pub chapter_list: Vec<String>,
    pub chapter_path_list: Vec<String>,
}

pub struct Message {
    app: tauri::AppHandle,
}

impl Message {
    pub fn new(app: tauri::AppHandle) -> Self {
        Message { app }
    }
    pub fn send(&self, msg: &str) {
        // println!("{}", msg);
        self.app.emit("message", msg).unwrap();
    }
    pub fn print(&self, msg: &str) {
        // println!("{}", msg);
        self.app.emit("image", msg).unwrap();
    }
}
