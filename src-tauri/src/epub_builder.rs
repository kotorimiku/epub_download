use anyhow::{anyhow, Result};
use chrono::Utc;
use std::fs::create_dir_all;
use std::path::Path;
use std::{collections::HashMap, fs::File, io::Write};
use zip::{write::SimpleFileOptions, CompressionMethod};

#[derive(Default, Debug)]
pub struct Metadata {
    pub title: String,
    pub creator: Option<String>,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub series: Option<String>,
    pub subject: Vec<String>,
    pub language: Option<String>,
    pub index: Option<usize>,
    pub identifier: Option<String>,
}

impl Metadata {
    pub fn new(
        title: &str,
        creator: Option<&str>,
        publisher: Option<&str>,
        description: Option<&str>,
        series: Option<&str>,
        subject: Vec<String>,
        language: Option<&str>,
        index: Option<usize>,
        identifier: Option<&str>,
    ) -> Self {
        let title = remove_invalid_xml_chars(&escape_epub_text(title));
        let creator = creator.map(|c| remove_invalid_xml_chars(&escape_epub_text(c)));
        let publisher = publisher.map(|p| remove_invalid_xml_chars(&escape_epub_text(p)));
        let description = description.map(|d| remove_invalid_xml_chars(&escape_epub_text(d)));
        let series = series.map(|s| remove_invalid_xml_chars(&escape_epub_text(s)));
        let language = language.map(|l| remove_invalid_xml_chars(&escape_epub_text(l)));
        let identifier = identifier.map(|i| remove_invalid_xml_chars(&escape_epub_text(i)));
        Self {
            title,
            creator,
            publisher,
            description,
            series,
            subject,
            language,
            index,
            identifier,
        }
    }
}

pub enum Body {
    Html(Vec<String>),
    Blocks(Vec<Vec<ContentBlock>>),
}

pub enum ContentBlock {
    Text(String),
    Image(usize),
}

pub struct EpubBuilder {
    metadata: Metadata,
    chapters: Body,
    chapter_titles: Vec<String>,
    images: Vec<Vec<u8>>,
    image_exts: Vec<String>,
    image_alts: Vec<String>,
    add_catalog: bool,
}

impl EpubBuilder {
    pub fn new(
        metadata: Metadata,
        chapters: Body,
        chapter_titles: Vec<String>,
        images: Vec<Vec<u8>>,
        image_exts: Vec<String>,
        image_alts: Vec<String>,
        add_catalog: bool,
    ) -> Self {
        let chapters = match chapters {
            Body::Blocks(blocks) => Body::Blocks(
                blocks
                    .into_iter()
                    .map(|chapter| {
                        chapter
                            .into_iter()
                            .map(|block| match block {
                                ContentBlock::Text(text) => ContentBlock::Text(
                                    remove_invalid_xml_chars(&escape_epub_text(&text)),
                                ),
                                ContentBlock::Image(_) => block,
                            })
                            .collect()
                    })
                    .collect(),
            ),
            Body::Html(html) => Body::Html(
                html.into_iter()
                    .map(|t| remove_invalid_xml_chars(&t))
                    .collect(),
            ),
        };
        let chapter_titles = chapter_titles
            .into_iter()
            .map(|t| remove_invalid_xml_chars(&escape_epub_text(&t)))
            .collect();
        EpubBuilder {
            metadata,
            chapters,
            chapter_titles,
            images,
            image_exts,
            image_alts,
            add_catalog,
        }
    }

    pub fn build_epub(&self) -> HashMap<String, Vec<u8>> {
        let mut epub = HashMap::new();
        // mimetype需要是第一个文件
        epub.insert(
            String::from("mimetype"),
            "application/epub+zip".as_bytes().to_vec(),
        );
        epub.insert(
            String::from("META-INF/container.xml"),
            self.build_container().as_bytes().to_vec(),
        );
        epub.insert(
            String::from("OEBPS/content.opf"),
            self.build_opf().as_bytes().to_vec(),
        );
        epub.insert(
            String::from("OEBPS/toc.ncx"),
            self.build_ncx().as_bytes().to_vec(),
        );
        epub.insert(
            String::from("OEBPS/Text/cover.xhtml"),
            self.build_cover_xhtml().as_bytes().to_vec(),
        );
        let html = match &self.chapters {
            Body::Html(html) => html,
            Body::Blocks(blocks) => &self.to_html(&blocks),
        };
        for i in 0..html.len() {
            epub.insert(
                format!("OEBPS/Text/{}.xhtml", self.num_fill(i + 1)),
                self.build_xhtml(&self.chapter_titles[i], &html[i])
                    .as_bytes()
                    .to_vec(),
            );
        }
        epub.insert(
            String::from("OEBPS/Text/nav.xhtml"),
            self.build_nav_xhtml().as_bytes().to_vec(),
        );
        for i in 0..self.image_exts.len() {
            let ext = &self.image_exts[i];
            epub.insert(
                format!("OEBPS/Images/{}.{}", self.num_fill(i), ext),
                self.images[i].clone(),
            );
        }
        if self.add_catalog {
            add_file(&mut epub, self.build_sgc_nav_css());
        }
        epub
    }

    fn to_html(&self, chapters: &Vec<Vec<ContentBlock>>) -> Vec<String> {
        chapters
            .into_iter()
            .map(|chapter| {
                chapter
                    .iter()
                    .map(|block| match block {
                        ContentBlock::Text(text) => {
                            if text.is_empty() {
                                String::from("<br/>")
                            } else {
                                format!("<p>{}</p>", text)
                            }
                        }
                        ContentBlock::Image(image) => format!(
                            "<img src=\"../Images/{}.{}\" alt=\"{}\" />",
                            self.num_fill(*image),
                            self.image_exts[*image],
                            self.image_alts[*image]
                        ),
                    })
                    .collect::<Vec<String>>()
                    .join("\n    ")
            })
            .collect()
    }

    pub fn save_file(&self, path: &Path) -> Result<()> {
        self.create_dir(path.parent())?;
        let mut file_map = self.build_epub();

        let zip_file = File::create(path).unwrap();
        let mut zip_writer = zip::ZipWriter::new(zip_file);
        // 设置默认的文件压缩选项
        let mimetype_options =
            SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        let mimetype = file_map.remove("mimetype").unwrap();
        zip_writer.start_file("mimetype", mimetype_options).unwrap();
        zip_writer.write_all(&mimetype).unwrap();

        for (file_name, file_data) in file_map {
            zip_writer.start_file(file_name, options).unwrap();
            zip_writer.write_all(&file_data).unwrap();
        }
        zip_writer.finish().unwrap();
        Ok(())
    }

    fn create_dir(&self, dir: Option<&Path>) -> Result<()> {
        if let Some(dir) = dir {
            if let Err(_) = create_dir_all(dir) {
                return Err(anyhow!("创建目录失败"));
            }
        }
        Ok(())
    }

    fn build_ncx(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE ncx PUBLIC "-//NISO//DTD ncx 2005-1//EN"
 "http://www.daisy.org/z3986/2005/ncx-2005-1.dtd">
<ncx version="2005-1" xmlns="http://www.daisy.org/z3986/2005/ncx/">
  <head>
    <meta name="dtb:depth" content="1" />
    <meta name="dtb:totalPageCount" content="0" />
    <meta name="dtb:maxPageNumber" content="0" />
  </head>
  <docTitle>
    <text>{}</text>
  </docTitle>
  <navMap>
    {}
  </navMap>
</ncx>"#,
            self.metadata.title,
            self.get_nav_xml()
        )
    }

    fn get_nav_xml(&self) -> String {
        let mut nav_map = Vec::new();
        for i in 0..self.chapter_titles.len() {
            nav_map.push(format!(
                r#"<navPoint id="navPoint-{}" playOrder="{}">
      <navLabel>
        <text>{}</text>
      </navLabel>
      <content src="{}" />
    </navPoint>"#,
                i + 1,
                i + 1,
                self.chapter_titles[i],
                format!("Text/{}.xhtml", self.num_fill(i + 1)),
            ));
        }
        nav_map.join("\n    ")
    }

    fn build_opf(&self) -> String {
        let metadata = self.get_metadata_xml();
        let manifest = self.get_manifest_xml();
        let spine = self.get_spine_xml();
        let guide = self.get_guide_xml();
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<package version="3.0" unique-identifier="BookId" xmlns="http://www.idpf.org/2007/opf">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:opf="http://www.idpf.org/2007/opf">
    {}
  </metadata>
  <manifest>
    {}
  </manifest>
  <spine toc="ncx">
    {}
  </spine>
  <guide>
    {}
  </guide>
</package>"#,
            metadata, manifest, spine, guide
        )
    }

    fn get_guide_xml(&self) -> String {
        let mut guide = Vec::new();
        guide.push("<reference href=\"Text/cover.xhtml\" title=\"Cover\" type=\"cover\"/>");
        guide.join("\n    ")
    }

    fn get_spine_xml(&self) -> String {
        let mut spine = Vec::new();
        spine.push(format!("<itemref idref=\"cover.xhtml\"/>"));
        // 添加目录页
        // spine.push(format!("<itemref idref=\"nav.xhtml\"/>"));
        for i in 0..self.chapter_titles.len() {
            spine.push(format!(
                "<itemref idref=\"x{}.xhtml\"/>",
                self.num_fill(i + 1)
            ));
        }
        spine.join("\n    ")
    }

    fn get_manifest_xml(&self) -> String {
        let mut manifest = Vec::new();
        manifest.push(format!("<item id=\"cover.xhtml\" href=\"Text/cover.xhtml\" media-type=\"application/xhtml+xml\"/>"));
        manifest.push(format!(
            "<item id=\"ncx\" href=\"toc.ncx\" media-type=\"application/x-dtbncx+xml\"/>"
        ));

        // text
        for i in 0..self.chapter_titles.len() {
            manifest.push(format!("<item id=\"x{}.xhtml\" href=\"Text/{}.xhtml\" media-type=\"application/xhtml+xml\"/>", self.num_fill(i + 1), self.num_fill(i + 1)));
        }

        // image
        for i in 0..self.images.len() {
            let ext = &self.image_exts[i];
            let media_type = if ext == "png" {
                "image/png"
            } else {
                "image/jpeg"
            };

            manifest.push(format!(
                "<item id=\"x{}.{}\" href=\"Images/{}.{}\" media-type=\"{}\"/>",
                self.num_fill(i),
                ext,
                self.num_fill(i),
                ext,
                media_type
            ));
        }
        manifest.push(r#"<item id="nav.xhtml" href="Text/nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>"#.to_string());
        if self.add_catalog {
            manifest.push(
                r#"<item id="sgc-nav.css" href="Styles/sgc-nav.css" media-type="text/css"/>"#
                    .to_string(),
            );
        }
        manifest.join("\n    ")
    }

    fn get_metadata_xml(&self) -> String {
        let mut metadata = Vec::new();
        metadata.push(format!("<dc:title>{}</dc:title>", self.metadata.title));
        if let Some(author) = &self.metadata.creator {
            metadata.push(format!("<dc:creator>{}</dc:creator>", author));
        }
        if let Some(publisher) = &self.metadata.publisher {
            metadata.push(format!("<dc:publisher>{}</dc:publisher>", publisher));
        }
        if let Some(description) = &self.metadata.description {
            metadata.push(format!("<dc:description>{}</dc:description>", description));
        }
        if let Some(language) = &self.metadata.language {
            metadata.push(format!("<dc:language>{}</dc:language>", language));
        }
        if let Some(identifier) = &self.metadata.identifier {
            metadata.push(format!(
                "<dc:identifier id=\"BookId\">{}</dc:identifier>",
                identifier
            ));
        }
        metadata.push(
            self.metadata
                .subject
                .iter()
                .map(|s| format!("<dc:subject>{}</dc:subject>", s))
                .collect::<Vec<String>>()
                .join("\n\t\t"),
        );
        metadata.push(format!(
            "<meta property=\"dcterms:modified\">{}</meta>",
            get_time()
        ));

        metadata.push(format!(
            "<meta name=\"cover\" content=\"x000.{}\"/>",
            self.image_exts[0]
        ));
        if let Some(series) = &self.metadata.series {
            metadata.push(format!(
                "<meta name=\"calibre:series\" content=\"{}\"/>",
                series
            ));
        }
        if let Some(index) = &self.metadata.index {
            metadata.push(format!(
                "<meta name=\"calibre:series_index\" content=\"{}\"/>",
                index
            ));
        }

        metadata.join("\n    ")
    }

    fn build_container(&self) -> String {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml" />
  </rootfiles>
</container>"#
            .to_string()
    }

    fn build_xhtml(&self, title: &str, body: &str) -> String {
        let title_tag = if title != "彩页" {
            format!("<h1>{}</h1>\n    ", title)
        } else {
            String::new()
        };
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>

<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
  <head>
    <title>{}</title>
    <style type="text/css">p{{text-indent:2em;}}</style>
  </head>
  <body>
    {}{}
  </body>
</html>"#,
            title, title_tag, body
        )
    }

    fn num_fill(&self, str: usize) -> String {
        format!("{:0>3}", str)
    }

    fn build_cover_xhtml(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>

<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head>
  <title>Cover</title>
</head>
<body>
  <div style="text-align: center; padding: 0pt; margin: 0pt;">
    <img src="../Images/000.{}" alt="cover" />
  </div>
</body>
</html>"#,
            self.image_exts[0]
        )
    }

    fn build_nav_xhtml(&self) -> String {
        let css = if self.add_catalog {
            format!(
                "\n  {}",
                r#"<link href="../Styles/sgc-nav.css" rel="stylesheet" type="text/css"/>"#
            )
        } else {
            String::new()
        };
        let mut nav_map = Vec::new();

        for i in 0..self.chapter_titles.len() {
            nav_map.push(format!(
                "<li><a href=\"{}.xhtml\">{}</a></li>",
                self.num_fill(i + 1),
                self.chapter_titles[i]
            ));
        }

        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>

<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" lang="en" xml:lang="en">
<head>
  <title>ePub NAV</title>
  <meta charset="utf-8"/>{}
</head>
<body epub:type="frontmatter">
  <nav epub:type="toc" id="toc" role="doc-toc">
    <h1>目录</h1>
    <ol>
      {}
    </ol>
  </nav>
</body>
</html>"#,
            css,
            nav_map.join("\n      ")
        )
    }

    fn build_sgc_nav_css(&self) -> (String, Vec<u8>) {
        let file_path = String::from("OEBPS/Styles/sgc-nav.css");
        (
            file_path,
            r#"nav#toc {
  font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
  padding: 20px;
  background-color: #f8f8f8; /* 浅灰色背景 */
  border-radius: 10px;
  box-shadow: 0px 4px 6px rgba(0, 0, 0, 0.1); /* 柔和的阴影 */
}

nav#toc h1 {
  font-size: 24px;
  color: #333;
  text-align: center;
  margin-bottom: 20px;
  font-weight: bold; /* 加粗 */
}

nav#toc ol {
  list-style-type: none;
  padding-left: 0;
}

nav#toc ol li {
  margin-bottom: 10px;
}

nav#toc ol li a {
  text-decoration: none;
  font-size: 18px;
  color: #555;
  padding: 6px;
  display: block;
  transition: background-color 0.3s, color 0.3s;
  border-radius: 5px;
}

nav#toc ol li a:hover {
  background-color: #d9d9d9;
  color: #000;
}
"#
            .as_bytes()
            .to_vec(),
        )
    }
}

fn add_file(epub: &mut HashMap<String, Vec<u8>>, file: (String, Vec<u8>)) {
    epub.insert(file.0, file.1);
}

fn get_time() -> String {
    // 获取当前 UTC 时间
    let now = Utc::now();

    // 格式化为 ISO 8601 格式 (YYYY-MM-DDThh:mm:ssZ)
    let iso8601_time = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    iso8601_time
}

pub fn escape_epub_text(input: &str) -> String {
    input
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
}

fn remove_invalid_xml_chars(input: &str) -> String {
    input
        .chars()
        .filter(|&c| match c as u32 {
            0x9 | 0xA | 0xD => true,
            0x20..=0xD7FF => true,
            0xE000..=0xFFFD => true,
            0x10000..=0x10FFFF => true,
            _ => false,
        })
        .collect()
}
