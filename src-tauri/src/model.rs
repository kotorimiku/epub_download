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
    pub url_vol: Option<String>,
}

pub struct Message {
    app: Option<tauri::AppHandle>,
}

impl Message {
    pub fn new(app: Option<tauri::AppHandle>) -> Self {
        Message { app }
    }
    pub fn send(&self, msg: &str) {
        if let Some(app) = &self.app {
            app.emit("message", msg).unwrap();
        }
        else {
            println!("{}", msg);
        }
    }
    pub fn print(&self, msg: &str) {
        if let Some(app) = &self.app {
            app.emit("image", msg).unwrap();
        }
        else {
            print!("{}", msg);
        }
    }
}
