use std::collections::HashSet;
use std::fs::{read_to_string, write};
use std::sync::{Mutex, OnceLock};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Config {
    pub output_path: String,
    pub add_number: bool,
    pub cookie: String,
    pub sleep_time: u64,
    pub url_base: String,
    #[serde(default)]
    pub add_catalog: bool,
    #[serde(default)]
    pub error_img: HashSet<String>,
}

impl Config {
    pub fn default() -> Config {
        Config {
            output_path: String::from("./"),
            add_number: false,
            cookie: String::from(""),
            sleep_time: 8,
            url_base: String::from("https://www.bilinovel.com"),
            add_catalog: false,
            error_img: HashSet::new(),
        }
    }
}

pub fn save_config() -> Result<(), Box<dyn std::error::Error>>{
    write("./config.json", serde_json::to_string_pretty(&CONFIG.get().unwrap())?)?;
    Ok(())
}

pub fn get_config() -> std::sync::MutexGuard<'static, Config> {
    CONFIG.get_or_init(|| {
        Mutex::new(read_to_string("./config.json")
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_else(|| {
                let config = Config::default();
                config
            }),
        )
    })
    .lock()
    .unwrap()
}

pub fn update_config(mut new_config: Config) {
    new_config.error_img = get_config().error_img.clone();
    if let Some(mutex) = CONFIG.get() {
        let mut config = mutex.lock().unwrap();
        let original_config = std::mem::replace(&mut *config, new_config);
        config.error_img = original_config.error_img;
    }
}

pub static CONFIG: OnceLock<Mutex<Config>> = OnceLock::new();
