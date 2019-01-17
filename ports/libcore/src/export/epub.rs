use super::{prelude::*, *};
use crate::{archive, storage};
use chrono::{offset::Utc, DateTime};
use std::fs::File;
use std::io::prelude::*;
use tera::{Context, Tera};
use uuid::Uuid;

pub struct Epub {
    pub platform: Platform,
    pub section: Section,
    pub uuid: String,
}

impl Epub {
    pub fn new(platform: Platform, section: Section) -> Self {
        let uuid = Uuid::new_v4().to_hyphenated().to_string();
        Self {
            platform,
            section,
            uuid,
        }
    }

    pub fn render_start_xhtml(&self) -> String {
        let tpl_s = r#"
<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" lang="en">
   <head>
      <title>{{ name }} - 关于</title>
      <link href="stylesheet.css" rel="stylesheet" type="text/css" />
   </head>
   <body>
      <h1>版权信息</h1>
      <p>图书名：{{ name }}</p>
      <p>
         来源于：<a href="{{ platform_url }}">{{ platform_name }}</a>
      </p>
      <p>操作人：{{ operator }}({{ version }})</p>
      <hr />
      <p>
         本图书由开源项目:
         <a href="https://manga.bluerain.io">MANGA-RS</a>
         生成，资源来自于第三方。
      </p>
      <strong>注：公开传播则意味着存在被版权方追究责任的风险。</strong>
   </body>
</html>
        "#
        .trim();
        let mut ctx = Context::new();
        ctx.insert("name", &self.section.name);
        ctx.insert("platform_url", &self.platform.url);
        ctx.insert("platform_name", &self.platform.name);
        ctx.insert("operator", "manga-bot");
        ctx.insert("version", &VERSION);
        Tera::one_off(&tpl_s, &ctx, false).unwrap()
    }

    pub fn render_page_html(&self, name: &str, src: &str) -> String {
        let tpl_s = r#"
<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml">
   <head>
      <title>{{ name }}</title>
      <link href="stylesheet.css" rel="stylesheet" type="text/css" />
   </head>
   <body class="album">
      <img class="albumimg" src="{{ img_src }}" />
   </body>
</html>
        "#
        .trim();
        let mut ctx = Context::new();
        ctx.insert("name", &name);
        ctx.insert("img_src", &src);
        Tera::one_off(&tpl_s, &ctx, false).unwrap()
    }

    pub fn render_metadata_opf(&self) -> String {
        let tpl_s = r#"
<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" unique-identifier="uuid_id" version="2.0">
   <metadata xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:opf="http://www.idpf.org/2007/opf">
      <dc:title>{{ title }}</dc:title>
      <dc:creator opf:role="aut" opf:file-as="MANGA-BOT">MANGA-BOT</dc:creator>
      <dc:identifier opf:scheme="uuid" id="uuid_id">{{ uuid }}</dc:identifier>
      <dc:publisher>manga.bluerain.io</dc:publisher>
      <dc:contributor opf:file-as="manga-rs" opf:role="bkp">manga-rs ({{ version }}) [https://manga.bluerain.io]</dc:contributor>
      <dc:date>{{ date_time }}</dc:date>
      <dc:language>eng</dc:language>
      <meta name="cover" content="cover" />
   </metadata>
   <manifest>
      <item href="toc.ncx" id="ncx" media-type="application/x-dtbncx+xml" />
      <item href="stylesheet.css" id="id33" media-type="text/css" />
      <item href="start.xhtml" id="start" media-type="application/xhtml+xml" />
      {% for p in plist %}
      <item href="{{ p.p }}.html" id="page{{ p.p }}" media-type="application/xhtml+xml" />
      <item href="{{ p.p }}.{{ p.extension }}" id="img{{ p.p }}" media-type="{{ p.mime }}" />
      {% endfor %}
      <item href="cover.{{ plist.0.extension }}" id="cover" media-type="{{ plist.0.mime }}" />
   </manifest>
   <spine toc="ncx">
      <itemref idref="start" />
      {% for p in plist %}
      <itemref idref="page{{ p.p }}" />
      {% endfor %}
   </spine>
   <guide />
</package>
        "#
            .trim();
        let mut ctx = Context::new();
        ctx.insert("title", &self.section.name);
        ctx.insert("uuid", &self.uuid);
        ctx.insert("plist", &self.section.page_list);
        ctx.insert("version", &VERSION);
        ctx.insert("date_time", &DateTime::from(Utc::now()).to_rfc3339());
        Tera::one_off(&tpl_s, &ctx, false).unwrap()
    }

    pub fn render_stylesheet(&self) -> String {
        r#"
* {
   padding: 0;
   margin: 0;
}

.album {
   background: #000000;
   height: 100%;
   text-align: center;
   vertical-align: top;
}

.albumimg {
   margin: 0;
   height: 100%;
   text-align: center;
   vertical-align: top;
}
        "#
        .trim()
        .to_string()
    }

    pub fn render_toc_ncx(&self) -> String {
        let tpl_s = r#"
<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1" xml:lang="en">
   <head>
      <meta content="{{ uuid }}" name="dtb:uid" />
      <meta content="2" name="dtb:depth" />
      <meta content="manga-rs" name="dtb:generator" />
      <meta content="0" name="dtb:totalPageCount" />
      <meta content="0" name="dtb:maxPageNumber" />
   </head>
   <docTitle>
      <text>{{ name }}</text>
   </docTitle>
   <navMap>
      <navPoint id="navPoint-00" playOrder="0">
         <navLabel>
            <text>关于</text>
         </navLabel>
         <content src="start.xhtml" />
      </navPoint>
      {% for p in plist %}
      <navPoint id="navPoint-{{ p.p }}" playOrder="{{ p.p }}">
         <navLabel>
            <text>{{ p.p }}P</text>
         </navLabel>
         <content src="{{ p.p }}.html" />
      </navPoint>
      {% endfor %}
   </navMap>
</ncx>
        "#
        .trim();
        let mut ctx = Context::new();
        ctx.insert("name", &self.section.name);
        ctx.insert("uuid", &self.uuid);
        ctx.insert("plist", &self.section.page_list);
        Tera::one_off(&tpl_s, &ctx, false).unwrap()
    }

    pub fn render_container_xml(&self) -> String {
        r#"
<?xml version="1.0" encoding="UTF-8"?>
<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container" version="1.0">
   <rootfiles>
      <rootfile full-path="metadata.opf" media-type="application/oebps-package+xml" />
   </rootfiles>
</container>
        "#
        .trim()
        .to_string()
    }
}

impl Exporter for Epub {
    fn save(&mut self, output_dir: &str) -> Result<String> {
        // 下载整个 Section 的资源
        storage::from_section(&mut self.section)?.finish();
        // 建立输出目录
        std::fs::create_dir_all(output_dir)?;
        // 建立缓存目录
        let cache_dir = format!("manga_res/{}/.cache", &self.section.name);
        std::fs::create_dir_all(&cache_dir)?;
        let meta_dir = format!("{}/META-INF", &cache_dir);
        std::fs::create_dir_all(&meta_dir)?;
        // 注入变量并输出 EPUB 结构
        // start.xhtml
        let mut start_xhtml = File::create(format!("{}/start.xhtml", &cache_dir))?;
        start_xhtml.write_all(self.render_start_xhtml().as_bytes())?;
        // 循环写入所有的图片页面和文件
        for page in &self.section.page_list {
            let img_name = format!("{}.{}", &page.p, &page.extension);
            let mut img_xhtml = File::create(format!("{}/{}.html", &cache_dir, page.p))?;
            {
                img_xhtml.write_all(
                    self.render_page_html(&page.p.to_string(), &img_name)
                        .as_bytes(),
                )?;
            }
            let origin_path = format!(
                "{}/{}/origins/{}",
                "manga_res", &self.section.name, &img_name
            );
            std::fs::copy(&origin_path, format!("{}/{}", &cache_dir, &img_name))?;
            // 复制第一张图为封面
            if page.p == 0 {
                std::fs::copy(
                    &origin_path,
                    format!("{}/{}", &cache_dir, format!("cover.{}", &page.extension)),
                )?;
            }
        }
        // 写入 metadata.opf
        let mut metadata = File::create(format!("{}/metadata.opf", &cache_dir))?;
        metadata.write_all(self.render_metadata_opf().as_bytes())?;
        // 写入 mimetype
        let mut mimetype = File::create(format!("{}/mimetype", &cache_dir))?;
        mimetype.write_all("application/epub+zip".as_bytes())?;
        // 写入 stylesheet.css
        let mut stylesheet = File::create(format!("{}/stylesheet.css", &cache_dir))?;
        stylesheet.write_all(self.render_stylesheet().as_bytes())?;
        // 写入 toc.ncx
        let mut toc = File::create(format!("{}/toc.ncx", &cache_dir))?;
        toc.write_all(self.render_toc_ncx().as_bytes())?;
        // 写入 META-INF/container.xml
        let mut container = File::create(format!("{}/container.xml", &meta_dir))?;
        container.write_all(self.render_container_xml().as_bytes())?;

        // 打包成 epub
        let dst_file = format!("{}/{}.epub", &output_dir, &self.section.name);
        archive::doit(&cache_dir, &dst_file)?;
        Ok(dst_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_epub() {
        let platform = Platform::new("动漫之家", "https://manhua.dmzj.com");
        let mut section = Section::new(
            "流浪猫的一生  第01话",
            "https://manhua.dmzj.com/liulangmaodeyisheng/81737.shtml#@page=1",
        );
        section.add_page(Page::new(0, "https://images.dmzj.com/l/%E6%B5%81%E6%B5%AA%E7%8C%AB%E7%9A%84%E4%B8%80%E7%94%9F/%E7%AC%AC01%E8%AF%9D/001.jpg"));
        section.add_page(Page::new(1, "https://images.dmzj.com/l/%E6%B5%81%E6%B5%AA%E7%8C%AB%E7%9A%84%E4%B8%80%E7%94%9F/%E7%AC%AC01%E8%AF%9D/002.jpg"));
        section.add_page(Page::new(2, "https://images.dmzj.com/l/%E6%B5%81%E6%B5%AA%E7%8C%AB%E7%9A%84%E4%B8%80%E7%94%9F/%E7%AC%AC01%E8%AF%9D/003.jpg"));
        let mut epub = Epub::new(platform, section);
        let dst_file = epub.save(crate::DEFAULT_OUTPUT_DIR).unwrap();
        assert!(std::path::Path::new(&dst_file).exists());
    }
}
