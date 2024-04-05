use chrono::{DateTime, FixedOffset, NaiveDateTime};
use pulldown_cmark::{Event, TextMergeStream};
use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;

// 处理带 yaml front matter 的 markdown 文件，而且文件名符合 "%Y%m%d%H%M%S" 格式
// 验证文件有效性，以及放入内存以减少磁盘操作
// 同时具备这些特征，才能被认为是有效 markdown：
//
// - is_file() == true
// - f.len() < 100 mib
// - 内容均为 utf-8 编码
//
// 当内容无效时，返回 none

fn get_date<P: AsRef<Path>>(path: P) -> Option<DateTime<FixedOffset>> {
    let file_stem = path.as_ref().file_stem().unwrap().to_str().unwrap();
    if let Ok(ndt) = NaiveDateTime::parse_from_str(file_stem, "%Y%m%d%H%M%S") {
        unsafe {
            let fixed_offset = FixedOffset::east_opt(8 * 3600).unwrap_unchecked();
            let dt = ndt.and_local_timezone(fixed_offset).unwrap();
            return Some(dt);
        }
    };
    None
}

// TODO 变量化 FixedOffset 的时区
#[derive(Debug)]
pub struct Page {
    pub date: DateTime<FixedOffset>,
    pub front_matter: serde_yaml::Value,
    pub md: String,
}

static RE: OnceLock<Regex> = OnceLock::new();

impl Page {
    pub fn new<P: AsRef<Path>>(path: P) -> Option<Page> {
        let mut f = File::open(&path).ok()?;
        let m = f.metadata().ok()?;

        if !m.is_file() {
            return None;
        }

        if m.len() > 100_000_000 {
            return None;
        }

        let mut buffer = String::new();
        f.read_to_string(&mut buffer).ok()?;

        let date = get_date(&path)?;

        let (front_matter, md) = Page::parse(&buffer);

        Some(Page {
            date,
            front_matter,
            md,
        })
    }

    pub fn md_html(&self) -> String {
        let parser = pulldown_cmark::Parser::new(&self.md);
        let mut html = String::new();
        pulldown_cmark::html::push_html(&mut html, parser);
        html
    }

    pub fn parse(content: &str) -> (serde_yaml::Value, String) {
        let mut fm = serde_yaml::Value::Null;
        let md;

        let re = RE.get_or_init(|| Regex::new(r#"^---\n([\s\S]*?)\n---"#).unwrap());

        if let Some(m) = re.find(content.trim()) {
            let t = content[m.start() + 4..m.end() - 4].trim();
            fm = match serde_yaml::from_str::<serde_yaml::Value>(t) {
                Ok(fm) => fm,
                Err(_) => {
                    println!("{}", content);
                    panic!()
                }
            };
            md = content[m.end()..].trim().to_string();
        } else {
            md = content.trim().to_string();
        }

        (fm, md)
    }

    pub fn title(&self) -> String {
        match self.md.lines().next() {
            Some(line) => {
                let parser = pulldown_cmark::Parser::new(line);
                let mut html = String::new();
                let iterator = TextMergeStream::new(parser);
                iterator.for_each(|e| {
                    if let Event::Text(t) = e {
                        html.push_str(&t)
                    }
                });

                html
            }
            None => "".to_owned(),
        }
    }

    pub fn id(&self) -> String {
        self.date.format("%Y%m%d%H%M%S").to_string()
    }
}

pub struct Posts {
    pages: Vec<Page>,
}

impl Posts {
    pub fn new<P: AsRef<Path>>(path: P) -> Option<Posts> {
        let rd = std::fs::read_dir(path).ok()?;
        let mut pages = Vec::new();
        rd.into_iter().for_each(|p| {
            let entry = p.unwrap();
            if let Some(page) = Page::new(entry.path()) {
                pages.push(page);
            }
        });

        if pages.len() == 0 {
            return None;
        }

        pages.sort_unstable_by(|a, b| b.date.cmp(&a.date));

        Some(Posts { pages })
    }
}

impl IntoIterator for Posts {
    type Item = Page;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.pages.into_iter()
    }
}
