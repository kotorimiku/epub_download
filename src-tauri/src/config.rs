use anyhow::Result;
use specta::Type;
use std::collections::HashSet;
use std::fs::{read_to_string, write};

const CONFIG_FILE: &str = "./config.json";

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub output: String,
    pub template: String,
    pub cookie: String,
    pub sleep_time: u32,
    pub base_url: String,
    #[serde(default)]
    pub add_catalog: bool,
    #[serde(default)]
    pub error_img: HashSet<String>,
}

impl Config {
    pub fn new() -> Config {
        Config {
            output: String::from("./"),
            template: "{{book_title}}-{{chapter_title}}".to_string(),
            cookie: String::from(""),
            sleep_time: 8,
            base_url: String::from("https://www.bilinovel.com"),
            add_catalog: false,
            error_img: HashSet::new(),
        }
    }

    pub fn save(&self) -> Result<()> {
        write(CONFIG_FILE, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn load() -> Config {
        read_to_string(CONFIG_FILE)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_else(|| {
                let config = Config::new();
                config
            })
    }
}
