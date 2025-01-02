use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct MetaData {
    pub title: String,
    pub creator: Option<String>,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub series: Option<String>,
    pub subject: Vec<String>,
    pub language: Option<String>,
    pub index: Option<usize>,
}

pub struct EpubBuilder {
    metadata: MetaData,
    text: Vec<String>,
    chapter_list: Vec<String>,
    img_data_list: Vec<Vec<u8>>,
    ext_list: Vec<String>,
}

impl EpubBuilder {
    pub fn new(
        metadata: MetaData,
        text: Vec<String>,
        chapter_list: Vec<String>,
        img_data_list: Vec<Vec<u8>>,
        ext_list: Vec<String>,
    ) -> Self {
        EpubBuilder {
            metadata,
            text,
            chapter_list,
            img_data_list,
            ext_list,
        }
    }

    pub fn build_epub(&self) -> HashMap<String, Vec<u8>> {
        let mut epub = HashMap::new();
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
        for i in 0..self.text.len() {
            epub.insert(
                format!("OEBPS/Text/{}.xhtml", self.num_fill(i + 1)),
                self.build_xhtml(&self.chapter_list[i], &self.text[i])
                    .as_bytes()
                    .to_vec(),
            );
        }
        for i in 0..self.ext_list.len() {
            let ext = self.ext_list[i].split(".").last().unwrap();
            epub.insert(
                format!("OEBPS/Images/{}.{}", self.num_fill(i), ext),
                self.img_data_list[i].clone(),
            );
        }
        epub
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
        for i in 0..self.chapter_list.len() {
            nav_map.push(format!(
                r#"<navPoint id="navPoint-{}" playOrder="{}">
        <navLabel>
            <text>{}</text>
        </navLabel>
    <content src="{}" />
</navPoint>"#,
                i + 1,
                i + 1,
                self.chapter_list[i],
                format!("Text/{}.xhtml", self.num_fill(i + 1)),
            ));
        }
        nav_map.join("\n\t")
    }

    fn build_opf(&self) -> String {
        let metadata = self.get_metadata_xml();
        let manifest = self.get_manifest_xml();
        let spine = self.get_spine_xml();
        let guide = self.get_guide_xml();
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/"
    unique-identifier="bookid" version="2.0">
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
        guide.join("\n\t\t")
    }

    fn get_spine_xml(&self) -> String {
        let mut spine = Vec::new();
        spine.push(format!("<itemref idref=\"cover.xhtml\"/>"));
        for i in 0..self.chapter_list.len() {
            spine.push(format!(
                "<itemref idref=\"{}.xhtml\"/>",
                self.num_fill(i + 1)
            ));
        }
        spine.join("\n\t\t")
    }

    fn get_manifest_xml(&self) -> String {
        let mut manifest = Vec::new();
        manifest.push(format!("<item id=\"cover.xhtml\" href=\"Text/cover.xhtml\" media-type=\"application/xhtml+xml\"/>"));
        manifest.push(format!(
            "<item id=\"ncx\" href=\"toc.ncx\" media-type=\"application/x-dtbncx+xml\"/>"
        ));

        // text
        for i in 0..self.chapter_list.len() {
            manifest.push(format!("<item id=\"{}.xhtml\" href=\"Text/{}.xhtml\" media-type=\"application/xhtml+xml\"/>", self.num_fill(i + 1), self.num_fill(i + 1)));
        }

        // image
        for i in 0..self.img_data_list.len() {
            let ext = self.ext_list[i].split('.').last().unwrap();
            manifest.push(format!(
                "<item id=\"{}.{}\" href=\"Images/{}.{}\" media-type=\"image/jpeg\"/>",
                self.num_fill(i),
                ext,
                self.num_fill(i),
                ext
            ));
        }
        manifest.join("\n\t\t")
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
        metadata.push(
            self.metadata
                .subject
                .iter()
                .map(|s| format!("<dc:subject>{}</dc:subject>", s))
                .collect::<Vec<String>>()
                .join("\n\t\t"),
        );
        metadata.push(format!("<meta name=\"cover\" content=\"000.jpg\"/>"));
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

        metadata.join("\n\t\t")
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
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.1//EN" "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd">
<html xmlns="http://www.w3.org/1999/xhtml">
    <head>
        <title>{}</title>
        <style type="text/css">p{{text-indent:2em;}}</style>
    </head>
    <body>
        <h1>{}</h1>
        {}
    </body>
</html>"#,
            title, title, body
        )
    }

    fn num_fill(&self, str: usize) -> String {
        format!("{:0>3}", str)
    }

    fn build_cover_xhtml(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.1//EN" "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd">
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
  <title>Cover</title>
</head>
<body>
  <div style="text-align: center; padding: 0pt; margin: 0pt;">
    <img src="../Images/000{}" alt="cover" />
  </div>
</body>
</html>"#,
            self.ext_list[0]
        )
    }
}
