pub mod bilinovel;
pub mod client;
pub mod error;
pub mod listener;
pub mod model;
pub mod paragraph_restorer;
pub mod parse;
pub mod secret;
pub mod utils;

pub use bilinovel::BiliNovel;
pub use client::BiliClient;
pub use error::Result;
pub use listener::MessageCallback;
pub use model::{Book, BookInfo, Content, Volume, VolumeInfo};
