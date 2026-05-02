use std::process;

use anyhow::Result;
use clap::Parser;

use crate::{
    config::Config,
    downloader::{Downloader, DownloaderConfig},
};

#[derive(Parser, Debug)]
#[command(version = "0.1", about, long_about = None)]
struct Args {
    #[arg(short, long, help = "书籍id")]
    book_id: String,

    #[arg(short, long, default_value_t = String::new(), help = "需要下载的卷数，下载多卷请使用,分隔或者连字符-，下载所有使用all")]
    volume: String,

    #[arg(short, long, help = "输出目录")]
    output: Option<String>,

    #[arg(long)]
    cookie: Option<String>,

    #[arg(
        short,
        long,
        help = "自定义命名，
书籍标题使用{{book_title}}，章节标题使用{{chapter_title}}，
章节编号使用{{chapter_number}}，章节编号前填0使用{{chapter_number:x}}，
输入 0 ，使用{{book_title}}-{{chapter_title}}，
输入 1 ，使用{{book_title}}-[{{chapter_number}}]{{chapter_title}}，
输入 2 ，使用[{{chapter_number}}]{{chapter_title}}，
输入 3 ，使用[{{chapter_number:2}}]{{chapter_title}}
"
    )]
    template: Option<String>,

    #[arg(long, help = "开启调试模式，输出更多日志")]
    debug: Option<bool>,
}

pub async fn run_cli() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let mut config = Config::default();
    if let Some(output) = args.output {
        config.output = output;
    }
    if let Some(cookie) = args.cookie {
        config.cookie = cookie;
    }
    if let Some(template) = args.template {
        config.template = template;
    }
    if let Some(debug) = args.debug {
        config.debug = debug;
    }

    let book = Downloader::new(DownloaderConfig {
        base_url: config.base_url.clone(),
        book_id: args.book_id,
        output: config.output.clone(),
        template: config.template.clone(),
        sleep_time: config.sleep_time,
        convert_simple_chinese: config.convert_simple_chinese,
        cookie: config.cookie.clone(),
        user_agent: config.user_agent.clone(),
        header_map: config.headers.clone(),
        add_catalog: config.add_catalog,
        error_img: config.error_img.clone(),
        app_handle: None,
        debug: config.debug,
    })
    .await?;

    if args.volume.is_empty() {
        println!("{}", book.book_info.title.unwrap());
        for i in 0..book.volume_infos.len() {
            println!("[{}] {:?}", i + 1, book.volume_infos[i].title);
        }
        process::exit(0);
    }

    if args.volume == "all" {
        book.download(1..=book.volume_infos.len().try_into().unwrap())
            .await?;
    } else if args.volume.contains(',') {
        let list: std::str::Split<'_, &str> = args.volume.split(",");
        let list = list.map(|s| s.parse::<u32>().unwrap());
        book.download(list).await?;
    } else if args.volume.contains('-') {
        let list = args.volume.split("-");
        let mut list = list.map(|s| s.parse::<u32>().unwrap());
        let start = list.next().unwrap();
        let end = list.next().unwrap();
        book.download(start..=end).await?;
    } else {
        book.download(vec![args.volume.parse::<u32>().unwrap()].into_iter())
            .await?;
    }
    Ok(())
}
