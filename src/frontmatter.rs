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

#[derive(Default, Clone)]
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

#[derive(Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_front_matter() {
        let content = "---\ntitle: My Post\ndescription: A test\ntags: rust, programming\ndate: 2024-01-15\n---\n\n# Body";
        let (meta, body) = parse_front_matter(content);
        assert_eq!(meta.title, Some("My Post".to_string()));
        assert_eq!(meta.description, Some("A test".to_string()));
        assert_eq!(meta.tags, vec!["rust", "programming"]);
        assert_eq!(meta.date, Some("2024-01-15".to_string()));
        assert_eq!(body, "# Body");
    }

    #[test]
    fn test_parse_html_front_matter() {
        let content = "<!--\ntitle: My Page\ndescription: A page\ndraft: true\n-->\n\n<h1>Hello</h1>";
        let (meta, body) = parse_front_matter(content);
        assert_eq!(meta.title, Some("My Page".to_string()));
        assert_eq!(meta.description, Some("A page".to_string()));
        assert!(meta.draft);
        assert_eq!(body, "<h1>Hello</h1>");
    }

    #[test]
    fn test_parse_no_front_matter() {
        let content = "# Just content\nNo front matter.";
        let (meta, body) = parse_front_matter(content);
        assert!(meta.title.is_none());
        assert!(meta.description.is_none());
        assert!(meta.tags.is_empty());
        assert!(!meta.draft);
        assert_eq!(body, content);
    }

    #[test]
    fn test_draft_flag() {
        let (meta, _) = parse_front_matter("---\ndraft: true\n---\n");
        assert!(meta.draft);
        let (meta, _) = parse_front_matter("---\ndraft: false\n---\n");
        assert!(!meta.draft);
        let (meta, _) = parse_front_matter("---\ndraft: True\n---\n");
        assert!(meta.draft); // case-insensitive
    }

    #[test]
    fn test_tags_are_lowercased_and_trimmed() {
        let (meta, _) = parse_front_matter("---\ntags: Rust, Web Dev , PYTHON\n---\n");
        assert_eq!(meta.tags, vec!["rust", "web dev", "python"]);
    }

    #[test]
    fn test_empty_tags_filtered() {
        let (meta, _) = parse_front_matter("---\ntags: rust,,, web\n---\n");
        assert_eq!(meta.tags, vec!["rust", "web"]);
    }

    #[test]
    fn test_last_edited_and_location() {
        let content = "---\ndate: 2024-01-01\nlast_edited: 2024-06-15\nlocation: Hong Kong\n---\n";
        let (meta, _) = parse_front_matter(content);
        assert_eq!(meta.date, Some("2024-01-01".to_string()));
        assert_eq!(meta.last_edited, Some("2024-06-15".to_string()));
        assert_eq!(meta.location, Some("Hong Kong".to_string()));
    }

    #[test]
    fn test_language_and_og_image() {
        let content = "---\nlanguage: zh\nog_image: /static/cover.jpg\n---\n";
        let (meta, _) = parse_front_matter(content);
        assert_eq!(meta.language, Some("zh".to_string()));
        assert_eq!(meta.og_image, Some("/static/cover.jpg".to_string()));
    }

    #[test]
    fn test_body_leading_whitespace_stripped() {
        let content = "---\ntitle: Test\n---\n\n\n# Heading";
        let (_, body) = parse_front_matter(content);
        assert_eq!(body, "# Heading");
    }

    #[test]
    fn test_unknown_fields_ignored() {
        let content = "---\ntitle: Test\nunknown_field: value\nanother: 123\n---\nbody";
        let (meta, body) = parse_front_matter(content);
        assert_eq!(meta.title, Some("Test".to_string()));
        assert_eq!(body, "body");
    }
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
