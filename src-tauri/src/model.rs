use serde::{self, Serialize, Deserialize};
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
type App = crate::event::Event;

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
                app.message(msg);
                return;
            }
        }

        println!("{}", msg);
    }

    pub fn print(&self, msg: &str) {
        #[cfg(feature = "gui")]
        {
            if let Some(app) = &self.app {
                app.message(msg);
                return;
            }
        }

        print!("{}", msg);
    }
}
