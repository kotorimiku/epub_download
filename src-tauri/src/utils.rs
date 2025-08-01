use std::io::Cursor;
use fast2s::convert;
use image::{ImageFormat, ImageReader};
use image::codecs::jpeg::JpegEncoder;
use semver::Version;

pub fn t2s(str: &str) -> String {
    // traditional_to_simplified(str).to_string()
    convert(str)
}

/// 移除文件名中的非法字符
pub fn remove_invalid_chars(filename: &str) -> String {
    let invalid_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    filename.chars().filter(|&c| !invalid_chars.contains(&c)).collect()
}

pub fn escape_epub_text(input: &str) -> String {
    input
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
}

/// 将非png和jpg的图片转为jpg
pub fn img_to_jpg(data: Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let format = image::guess_format(&data)?;
    let img = match format {
        ImageFormat::Png => data,
        ImageFormat::Jpeg => data,
        _ => {
            let img = ImageReader::new(Cursor::new(data)).with_guessed_format()?.decode()?;
            let mut output = Vec::new();
            let mut encoder = JpegEncoder::new(&mut output);
            encoder.encode_image(&img)?;
            output
        }
    };
    Ok(img)
}

pub fn is_newer_version(local: &str, remote: &str) -> bool {
    let local_ver = Version::parse(local).unwrap_or_else(|_| Version::new(0,0,0));
    let remote_ver = Version::parse(remote.trim_start_matches('v')).unwrap_or_else(|_| Version::new(0,0,0));
    remote_ver > local_ver
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_t2s() {
        println!("{}", t2s("妳"));
        println!("{}", convert("妳"));
    }
}