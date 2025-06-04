use serde::{self, Serialize, Deserialize};
use specta::Type;

#[cfg(feature = "gui")]
use tauri::Emitter;

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

#[derive(Debug)]
pub enum Content {
    Text(String),
    Image(String),
}

#[cfg(feature = "gui")]
type App = tauri::AppHandle;

#[cfg(not(feature = "gui"))]
type App = ();

pub struct Message {
    #[allow(dead_code)]
    app: Option<App>,
}

impl Message {
    pub fn new(app: Option<App>) -> Self {
        Message { app: app }
    }

    pub fn send(&self, msg: &str) {
        #[cfg(feature = "gui")]
        {
            if let Some(app) = &self.app {
                app.emit("message", msg).unwrap();
                return;
            }
        }

        println!("{}", msg);
    }

    pub fn print(&self, msg: &str) {
        #[cfg(feature = "gui")]
        {
            if let Some(app) = &self.app {
                app.emit("image", msg).unwrap();
                return;
            }
        }

        print!("{}", msg);
    }
}
