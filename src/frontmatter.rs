use serde::Deserialize;
use std::path::PathBuf;

fn default_posts_per_page() -> usize { 10 }
fn default_webp_quality() -> u8 { 80 }
fn default_locale() -> String { "en_US".to_string() }

#[derive(Deserialize)]
pub struct SiteConfig {
    pub title: String,
    pub base_url: String,
    pub author: String,
    pub description: String,
    #[serde(default = "default_posts_per_page")]
    pub posts_per_page: usize,
    #[serde(default)]
    pub optimize_images: bool,
    #[serde(default = "default_webp_quality")]
    pub webp_quality: u8,
    #[serde(default)]
    pub max_image_width: Option<u32>,
    #[serde(default)]
    pub og_image: Option<String>,
    #[serde(default = "default_locale")]
    pub locale: String,
}

#[derive(Default)]
pub struct PageMeta {
    pub title: Option<String>,
    pub description: Option<String>,
    pub date: Option<String>,
    pub last_edited: Option<String>,
    pub tags: Vec<String>,
    pub location: Option<String>,
    pub language: Option<String>,
    pub og_image: Option<String>,
    pub draft: bool,
}

pub struct PageInfo {
    pub relative_dir: PathBuf,
    pub out_filename: String,
    pub page_url: String,
    pub full_url: String,
    pub meta: PageMeta,
    pub body: String,
    pub is_md: bool,
}

fn parse_meta_lines(text: &str) -> PageMeta {
    let mut meta = PageMeta::default();
    for line in text.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let value = value.trim();
            match key.trim() {
                "title" => meta.title = Some(value.to_string()),
                "description" => meta.description = Some(value.to_string()),
                "date" => meta.date = Some(value.to_string()),
                "last_edited" => meta.last_edited = Some(value.to_string()),
                "location" => meta.location = Some(value.to_string()),
                "language" => meta.language = Some(value.to_string()),
                "og_image" => meta.og_image = Some(value.to_string()),
                "draft" => meta.draft = value.eq_ignore_ascii_case("true"),
                "tags" => {
                    meta.tags = value
                        .split(',')
                        .map(|t| t.trim().to_lowercase())
                        .filter(|t| !t.is_empty())
                        .collect()
                }
                _ => {}
            }
        }
    }
    meta
}

pub fn parse_front_matter(content: &str) -> (PageMeta, String) {
    if let Some(rest) = content.strip_prefix("---") {
        let rest = rest.trim_start_matches('\n');
        if let Some(end) = rest.find("\n---") {
            let meta = parse_meta_lines(&rest[..end]);
            let body = rest[end + 4..].trim_start_matches('\n').to_string();
            return (meta, body);
        }
        eprintln!("[WARNING] Front matter opening '---' found but closing '---' is missing");
    }

    if content.starts_with("<!--") {
        if let Some(end) = content.find("-->") {
            let meta = parse_meta_lines(&content[4..end]);
            let body = content[end + 3..].trim_start_matches('\n').to_string();
            return (meta, body);
        }
        eprintln!("[WARNING] Front matter opening '<!--' found but closing '-->' is missing");
    }

    (PageMeta::default(), content.to_string())
}
