use anyhow::Result;
use specta::Type;
use std::collections::HashSet;
use std::fs::{read_to_string, write};

const CONFIG_FILE: &str = "./config.json";

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "default_output")]
    pub output: String,
    #[serde(default = "default_template")]
    pub template: String,
    #[serde(default = "default_cookie")]
    pub cookie: String,
    #[serde(default = "default_sleep_time")]
    pub sleep_time: u32,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_add_catalog")]
    pub add_catalog: bool,
    #[serde(default = "default_error_img")]
    pub error_img: HashSet<String>,
    #[serde(default = "default_auto_check_update")]
    pub auto_check_update: bool,
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
            auto_check_update: true,
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

fn default_output() -> String {
    String::from("./")
}

fn default_template() -> String {
    String::from("{{book_title}}-{{chapter_title}}")
}

fn default_cookie() -> String {
    String::from("")
}

fn default_sleep_time() -> u32 {
    8
}

fn default_base_url() -> String {
    String::from("https://www.bilinovel.com")
}

fn default_add_catalog() -> bool {
    false
}

fn default_error_img() -> HashSet<String> {
    const ERROR_IMG: [&str; 7] = [
        "https://www.xlcx996.xyz/image/novel/sister01.jpg", // 3273/167199.html
        "「<img",                                           // 1744/180492.html
        "https://img3.readpai.com/3/3275/241359/263728.jpg", // 3275/241359.html
        "https://cdn-img.beixibaobao.cn/images/2vp7.png",   // 3305/168116_2.html
        "https://s6.jpg.cm/2022/07/12/Pn4pQS.jpg",          // 3342/169533_2.html
        "https://img1.imgtp.com/2022/07/26/S3ooRdwC.png",   // 3342/169525.html
        "https://img1.imgtp.com/2022/07/27/3kRju45s.png",   // 3342/169587_3.html
    ];
    ERROR_IMG.iter().map(|s| s.to_string()).collect()
}

fn default_auto_check_update() -> bool {
    true
}
