use crate::frontmatter::{PageInfo, SiteConfig};
use crate::highlight::convert_callouts;
use crate::utils::{format_date, format_rfc822, html_escape, json_escape, slugify, xml_escape};
use pulldown_cmark::{html as cm_html, Options, Parser};

pub fn generate_breadcrumb_json_ld(crumbs: &[(&str, &str)]) -> String {
    if crumbs.len() < 2 {
        return String::new();
    }
    let entries: String = crumbs
        .iter()
        .enumerate()
        .map(|(i, (name, url))| {
            format!(
                "{{\"@type\":\"ListItem\",\"position\":{},\"name\":\"{}\",\"item\":\"{}\"}}",
                i + 1,
                json_escape(name),
                json_escape(url)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "<script type=\"application/ld+json\">{{\"@context\":\"https://schema.org\",\
        \"@type\":\"BreadcrumbList\",\"itemListElement\":[{}]}}</script>",
        entries
    )
}

pub fn generate_article_json_ld(
    title: &str,
    description: &str,
    url: &str,
    image: &str,
    published: &str,
    modified: Option<&str>,
    author: &str,
) -> String {
    let modified_field = modified
        .filter(|&m| m != published)
        .map(|m| format!(",\"dateModified\":\"{}\"", json_escape(m)))
        .unwrap_or_default();
    let image_field = if !image.is_empty() {
        format!(",\"image\":\"{}\"", json_escape(image))
    } else {
        String::new()
    };
    format!(
        "<script type=\"application/ld+json\">{{\"@context\":\"https://schema.org\",\
        \"@type\":\"BlogPosting\",\"headline\":\"{}\",\"description\":\"{}\",\
        \"url\":\"{}\",\"datePublished\":\"{}\"{}{},\
        \"author\":{{\"@type\":\"Person\",\"name\":\"{}\"}}}}</script>",
        json_escape(title), json_escape(description), json_escape(url),
        json_escape(published), modified_field, image_field, json_escape(author)
    )
}

pub fn apply_article_meta(html: &str, published: &str, modified: Option<&str>, author: &str) -> String {
    let article_tags = format!(
        "<meta property=\"og:type\" content=\"article\" />\n\
        <meta property=\"article:published_time\" content=\"{published}\" />\n\
        {modified_tag}\
        <meta property=\"article:author\" content=\"{author}\" />",
        modified_tag = modified
            .filter(|&m| m != published)
            .map(|m| format!("<meta property=\"article:modified_time\" content=\"{m}\" />\n"))
            .unwrap_or_default(),
    );
    html.replace("<meta property=\"og:type\" content=\"website\" />", &article_tags)
}

pub fn add_active_id_to_navbar(html: &str, page_name: &str) -> String {
    let search = format!("href=\"/{}/\"", page_name);
    let replace = format!("href=\"/{}/\" id=\"active\"", page_name);
    html.replace(&search, &replace)
}

#[allow(clippy::too_many_arguments)]
pub fn render_page(
    base_template: &str,
    title: &str,
    description: &str,
    keywords: &str,
    full_url: &str,
    og_image: &str,
    current_page_name: &str,
    content: &str,
    lang: &str,
) -> String {
    let rendered = base_template
        .replace("{{title}}", title)
        .replace("{{description}}", description)
        .replace("{{keywords}}", keywords)
        .replace("{{page_url}}", full_url)
        .replace("{{og_image}}", og_image)
        .replace("{{lang}}", lang);
    let rendered = add_active_id_to_navbar(&rendered, current_page_name);
    rendered.replace(
        "<main></main>",
        &format!("<main><div class=\"content\">{content}</div></main>"),
    )
}

pub fn generate_tag_nav(all_tags: &[String], active_tag: Option<&str>, section_url: &str, tag_base_url: &str) -> String {
    if all_tags.is_empty() {
        return String::new();
    }
    let all_class = if active_tag.is_none() {
        "tag-filter__btn tag-filter__btn--active"
    } else {
        "tag-filter__btn"
    };
    let mut html = format!("<a href=\"{section_url}\" class=\"{all_class}\">all</a>");
    for tag in all_tags {
        let slug = slugify(tag);
        let is_active = active_tag == Some(slug.as_str());
        let class = if is_active {
            "tag-filter__btn tag-filter__btn--active"
        } else {
            "tag-filter__btn"
        };
        html.push_str(&format!(
            "<a href=\"{tag_base_url}{slug}/\" class=\"{class}\">{}</a>", html_escape(tag)
        ));
    }
    format!("<nav class=\"tag-filters\">{html}</nav>")
}

pub fn section_url_for_pages(pages: &[&PageInfo]) -> String {
    for p in pages {
        if let Some(first) = p.relative_dir.components().next() {
            let dir = first.as_os_str().to_string_lossy();
            return format!("/{dir}/");
        }
    }
    "/".to_string()
}

pub fn generate_post_list(pages: &[&PageInfo]) -> String {
    if pages.is_empty() {
        return String::new();
    }
    let items: String = pages
        .iter()
        .map(|p| {
            let title = html_escape(p.meta.title.as_deref().unwrap_or(&p.out_filename));
            let date = p
                .meta
                .date
                .as_deref()
                .map(|d| format!("<time class=\"post-list__date\">{}</time>", format_date(d)))
                .unwrap_or_default();
            let location = p
                .meta
                .location
                .as_deref()
                .map(|l| format!("<span class=\"post-list__location\">{}</span>", html_escape(l)))
                .unwrap_or_default();
            let tag_badges = if p.meta.tags.is_empty() {
                String::new()
            } else {
                let badges: String = p
                    .meta
                    .tags
                    .iter()
                    .map(|t| format!("<span class=\"post-list__tag\">{}</span>", html_escape(t)))
                    .collect();
                format!("<div class=\"post-list__tags\">{badges}</div>")
            };
            format!(
                "<li class=\"post-list__item\"><a href=\"{}\">\
                <div class=\"post-list__headline\"><h2>{title}</h2>{date}</div>\
                <div class=\"post-list__meta\">{tag_badges}{location}</div>\
                </a></li>",
                p.page_url
            )
        })
        .collect();
    format!("<ul class=\"post-list\">{items}</ul>")
}

pub fn generate_pagination_nav(current: usize, total: usize, section_url: &str) -> String {
    if total <= 1 {
        return String::new();
    }
    let nums: String = (1..=total)
        .map(|n| {
            if n == current {
                format!("<span class=\"pagination__num pagination__num--current\">{n}</span>")
            } else {
                let url = if n == 1 {
                    section_url.to_string()
                } else {
                    format!("{section_url}{n}/")
                };
                format!("<a href=\"{url}\" class=\"pagination__num\">{n}</a>")
            }
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<nav class=\"pagination\">{nums}</nav>")
}

pub fn generate_related_articles(current: &PageInfo, all_pages: &[PageInfo]) -> String {
    if current.meta.tags.is_empty() {
        return String::new();
    }
    let mut scored: Vec<(usize, &PageInfo)> = all_pages
        .iter()
        .filter_map(|p| {
            if p.out_filename == "index.html" || p.page_url == current.page_url {
                return None;
            }
            let shared = p.meta.tags.iter().filter(|t| current.meta.tags.contains(*t)).count();
            if shared > 0 { Some((shared, p)) } else { None }
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.meta.date.cmp(&a.1.meta.date)));
    scored.truncate(3);
    if scored.is_empty() {
        return String::new();
    }
    let items: String = scored
        .iter()
        .map(|(_, p)| {
            let title = html_escape(p.meta.title.as_deref().unwrap_or(&p.out_filename));
            let date = p.meta.date.as_deref()
                .map(|d| format!(" - <span class=\"related__date\">{}</span>", format_date(d)))
                .unwrap_or_default();
            format!("<li><a href=\"{}\">{title}</a>{date}</li>", p.page_url)
        })
        .collect();
    format!("<hr class=\"divider\" /><h3>Related Articles</h3><ul>{items}</ul>")
}

pub fn generate_robots_txt(base_url: &str) -> String {
    let base = base_url.trim().trim_end_matches('/');
    format!("User-agent: *\nAllow: /\nSitemap: {base}/sitemap.xml\n")
}

pub fn generate_sitemap(base_url: &str, urls: &[(String, Option<String>)]) -> String {
    let base = xml_escape(base_url.trim().trim_end_matches('/'));
    let entries = urls
        .iter()
        .map(|(url, date)| {
            let lastmod = date
                .as_deref()
                .map(|d| format!("<lastmod>{d}</lastmod>"))
                .unwrap_or_default();
            format!("<url><loc>{base}{}</loc>{lastmod}</url>", xml_escape(url))
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n{entries}\n</urlset>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontmatter::{PageInfo, PageMeta, SiteConfig};
    use std::path::PathBuf;

    fn make_config() -> SiteConfig {
        SiteConfig {
            title: "Test Site".to_string(),
            base_url: "https://example.com".to_string(),
            author: "Test Author".to_string(),
            description: "Test description".to_string(),
            posts_per_page: 10,
            optimize_images: false,
            webp_quality: 80,
            max_image_width: None,
            og_image: None,
            locale: "en_US".to_string(),
        }
    }

    fn make_page(title: &str, url: &str, date: Option<&str>, tags: &[&str]) -> PageInfo {
        PageInfo {
            relative_dir: PathBuf::new(),
            out_filename: format!("{}.html", url.trim_matches('/')),
            page_url: url.to_string(),
            full_url: format!("https://example.com{url}"),
            meta: PageMeta {
                title: Some(title.to_string()),
                date: date.map(|s| s.to_string()),
                tags: tags.iter().map(|t| t.to_string()).collect(),
                ..Default::default()
            },
            body: format!("Body of {title}"),
            is_md: false,
        }
    }

    // --- breadcrumb ---

    #[test]
    fn test_breadcrumb_basic() {
        let result = generate_breadcrumb_json_ld(&[
            ("Home", "https://example.com/"),
            ("Writing", "https://example.com/writing/"),
        ]);
        assert!(result.contains("BreadcrumbList"));
        assert!(result.contains("\"position\":1"));
        assert!(result.contains("\"position\":2"));
        assert!(result.contains("Home"));
        assert!(result.contains("Writing"));
    }

    #[test]
    fn test_breadcrumb_requires_at_least_two_items() {
        assert!(generate_breadcrumb_json_ld(&[]).is_empty());
        assert!(generate_breadcrumb_json_ld(&[("Home", "https://example.com/")]).is_empty());
    }

    #[test]
    fn test_breadcrumb_escapes_special_chars() {
        let result = generate_breadcrumb_json_ld(&[
            ("Home", "https://example.com/"),
            ("Say \"hello\"", "https://example.com/hi/"),
        ]);
        assert!(result.contains("Say \\\"hello\\\""));
    }

    #[test]
    fn test_breadcrumb_valid_json() {
        let result = generate_breadcrumb_json_ld(&[
            ("Home", "https://example.com/"),
            ("Section", "https://example.com/section/"),
            ("Article", "https://example.com/section/article/"),
        ]);
        let inner = result
            .trim_start_matches("<script type=\"application/ld+json\">")
            .trim_end_matches("</script>");
        assert!(serde_json::from_str::<serde_json::Value>(inner).is_ok(), "invalid JSON: {inner}");
    }

    // --- article json-ld ---

    #[test]
    fn test_article_json_ld_basic() {
        let result = generate_article_json_ld(
            "My Article", "A description",
            "https://example.com/article/",
            "https://example.com/image.webp",
            "2024-01-15", None, "Author Name",
        );
        assert!(result.contains("BlogPosting"));
        assert!(result.contains("My Article"));
        assert!(result.contains("2024-01-15"));
        assert!(result.contains("Author Name"));
        assert!(!result.contains("dateModified"));
    }

    #[test]
    fn test_article_json_ld_omits_modified_when_same_as_published() {
        let result = generate_article_json_ld(
            "Title", "Desc", "https://example.com/", "",
            "2024-01-15", Some("2024-01-15"), "Author",
        );
        assert!(!result.contains("dateModified"));
    }

    #[test]
    fn test_article_json_ld_includes_modified_when_different() {
        let result = generate_article_json_ld(
            "Title", "Desc", "https://example.com/", "",
            "2024-01-15", Some("2024-06-01"), "Author",
        );
        assert!(result.contains("dateModified"));
        assert!(result.contains("2024-06-01"));
    }

    #[test]
    fn test_article_json_ld_omits_image_when_empty() {
        let result = generate_article_json_ld(
            "Title", "Desc", "https://example.com/", "",
            "2024-01-15", None, "Author",
        );
        assert!(!result.contains("image"));
    }

    #[test]
    fn test_article_json_ld_escapes_special_chars() {
        let result = generate_article_json_ld(
            "Title with \"quotes\"", "Desc & more",
            "https://example.com/", "", "2024-01-15", None, "Author",
        );
        assert!(result.contains("Title with \\\"quotes\\\""));
        assert!(result.contains("Desc & more")); // & is valid in JSON, not escaped
    }

    #[test]
    fn test_article_json_ld_valid_json() {
        let result = generate_article_json_ld(
            "Title", "Desc", "https://example.com/article/",
            "https://example.com/img.webp",
            "2024-01-15", Some("2024-06-01"), "Author Name",
        );
        let inner = result
            .trim_start_matches("<script type=\"application/ld+json\">")
            .trim_end_matches("</script>");
        assert!(serde_json::from_str::<serde_json::Value>(inner).is_ok(), "invalid JSON: {inner}");
    }

    // --- apply_article_meta ---

    #[test]
    fn test_apply_article_meta_switches_og_type() {
        let html = r#"<meta property="og:type" content="website" />"#;
        let result = apply_article_meta(html, "2024-01-15", None, "Author");
        assert!(result.contains(r#"og:type" content="article""#));
        assert!(!result.contains(r#"og:type" content="website""#));
        assert!(result.contains("article:published_time"));
    }

    #[test]
    fn test_apply_article_meta_omits_modified_when_same() {
        let html = r#"<meta property="og:type" content="website" />"#;
        let result = apply_article_meta(html, "2024-01-15", Some("2024-01-15"), "Author");
        assert!(!result.contains("article:modified_time"));
    }

    #[test]
    fn test_apply_article_meta_includes_modified_when_different() {
        let html = r#"<meta property="og:type" content="website" />"#;
        let result = apply_article_meta(html, "2024-01-15", Some("2024-06-01"), "Author");
        assert!(result.contains("article:modified_time"));
        assert!(result.contains("2024-06-01"));
    }

    // --- pagination ---

    #[test]
    fn test_pagination_single_page_is_empty() {
        assert!(generate_pagination_nav(1, 1, "/writing/").is_empty());
        assert!(generate_pagination_nav(1, 0, "/writing/").is_empty());
    }

    #[test]
    fn test_pagination_marks_current_page() {
        let nav = generate_pagination_nav(2, 3, "/writing/");
        assert!(nav.contains("pagination__num--current"));
    }

    #[test]
    fn test_pagination_page_one_url_is_section_root() {
        let nav = generate_pagination_nav(2, 3, "/writing/");
        assert!(nav.contains("href=\"/writing/\""));
        assert!(nav.contains("href=\"/writing/3/\""));
    }

    // --- tag nav ---

    #[test]
    fn test_tag_nav_empty_returns_empty() {
        assert!(generate_tag_nav(&[], None, "/writing/", "/writing/tags/").is_empty());
    }

    #[test]
    fn test_tag_nav_active_tag_marked() {
        let tags = vec!["rust".to_string(), "web".to_string()];
        let nav = generate_tag_nav(&tags, Some("rust"), "/writing/", "/writing/tags/");
        assert!(nav.contains("tag-filter__btn--active"));
        assert!(nav.contains("/writing/tags/rust/"));
        assert!(nav.contains("/writing/tags/web/"));
    }

    #[test]
    fn test_tag_nav_no_active_marks_all() {
        let tags = vec!["rust".to_string()];
        let nav = generate_tag_nav(&tags, None, "/writing/", "/writing/tags/");
        assert!(nav.contains("tag-filter__btn--active"));
    }

    // --- related articles ---

    #[test]
    fn test_related_articles_by_shared_tags() {
        let current = make_page("Current", "/current/", Some("2024-01-01"), &["rust", "web"]);
        let related = make_page("Related", "/related/", Some("2024-01-02"), &["rust"]);
        let unrelated = make_page("Unrelated", "/unrelated/", Some("2024-01-03"), &["python"]);
        let all = vec![current.clone(), related, unrelated];
        let result = generate_related_articles(&current, &all);
        assert!(result.contains("Related"));
        assert!(!result.contains("Unrelated"));
        assert!(!result.contains("Current")); // excludes self
    }

    #[test]
    fn test_related_articles_empty_when_no_tags() {
        let current = make_page("Current", "/current/", None, &[]);
        let all = vec![current.clone()];
        assert!(generate_related_articles(&current, &all).is_empty());
    }

    #[test]
    fn test_related_articles_capped_at_three() {
        let current = make_page("Current", "/c/", None, &["rust"]);
        let others: Vec<PageInfo> = (0..5)
            .map(|i| make_page(&format!("Post {i}"), &format!("/post-{i}/"), None, &["rust"]))
            .collect();
        let mut all = vec![current.clone()];
        all.extend(others);
        let result = generate_related_articles(&current, &all);
        assert_eq!(result.matches("<li>").count(), 3);
    }

    // --- robots.txt ---

    #[test]
    fn test_robots_txt_contains_required_fields() {
        let result = generate_robots_txt("https://example.com");
        assert!(result.contains("User-agent: *"));
        assert!(result.contains("Allow: /"));
        assert!(result.contains("Sitemap: https://example.com/sitemap.xml"));
    }

    #[test]
    fn test_robots_txt_strips_trailing_slash_from_base() {
        let result = generate_robots_txt("https://example.com/");
        assert!(result.contains("Sitemap: https://example.com/sitemap.xml"));
        assert!(!result.contains("sitemap.xml/"));
    }

    // --- sitemap ---

    #[test]
    fn test_sitemap_basic() {
        let urls = vec![
            ("/".to_string(), None),
            ("/about/".to_string(), Some("2024-01-15".to_string())),
        ];
        let sitemap = generate_sitemap("https://example.com", &urls);
        assert!(sitemap.contains("<loc>https://example.com/</loc>"));
        assert!(sitemap.contains("<loc>https://example.com/about/</loc>"));
        assert!(sitemap.contains("<lastmod>2024-01-15</lastmod>"));
    }

    #[test]
    fn test_sitemap_no_empty_lastmod() {
        let urls = vec![("/".to_string(), None)];
        let sitemap = generate_sitemap("https://example.com", &urls);
        assert!(!sitemap.contains("<lastmod>"));
    }

    #[test]
    fn test_sitemap_strips_trailing_slash_from_base() {
        let urls = vec![("/about/".to_string(), None)];
        let sitemap = generate_sitemap("https://example.com/", &urls);
        assert!(sitemap.contains("<loc>https://example.com/about/</loc>"));
        assert!(!sitemap.contains("<loc>https://example.com//about/</loc>"));
    }

    #[test]
    fn test_sitemap_section_index_with_lastmod() {
        let urls = vec![
            ("/writing/".to_string(), Some("2024-06-01".to_string())),
        ];
        let sitemap = generate_sitemap("https://example.com", &urls);
        assert!(sitemap.contains("<lastmod>2024-06-01</lastmod>"));
        assert!(sitemap.contains("<loc>https://example.com/writing/</loc>"));
    }

    #[test]
    fn test_sitemap_article_lastmod_from_last_edited() {
        let urls = vec![
            ("/writing/my-post/".to_string(), Some("2024-09-15".to_string())),
        ];
        let sitemap = generate_sitemap("https://example.com", &urls);
        assert!(sitemap.contains("<lastmod>2024-09-15</lastmod>"));
    }

    #[test]
    fn test_sitemap_escapes_special_chars_in_base_url() {
        let urls = vec![("/".to_string(), None)];
        let sitemap = generate_sitemap("https://example.com&test", &urls);
        assert!(sitemap.contains("https://example.com&amp;test"));
    }

    // --- rss ---

    #[test]
    fn test_rss_only_includes_dated_non_index_pages() {
        let config = make_config();
        let dated = make_page("Post", "/post/", Some("2024-01-15"), &[]);
        let undated = make_page("Page", "/page/", None, &[]);
        let mut index = make_page("Index", "/writing/", Some("2024-01-15"), &[]);
        index.out_filename = "index.html".to_string();
        let pages = vec![dated, undated, index];
        let rss = generate_rss(&config, &pages);
        assert!(rss.contains("<title>Post</title>"));
        assert!(!rss.contains("<title>Page</title>"));
        assert!(!rss.contains("<title>Index</title>"));
    }

    #[test]
    fn test_rss_escapes_xml_special_chars() {
        let config = make_config();
        let mut page = make_page("Post & Article", "/post/", Some("2024-01-15"), &[]);
        page.meta.description = Some("A & B < C".to_string());
        let rss = generate_rss(&config, &vec![page]);
        assert!(rss.contains("Post &amp; Article"));
        assert!(rss.contains("A &amp; B &lt; C"));
    }
}

pub fn generate_rss(config: &SiteConfig, pages: &[PageInfo]) -> String {
    let base = xml_escape(config.base_url.trim().trim_end_matches('/'));
    let items: String = pages
        .iter()
        .filter(|p| p.meta.date.is_some() && p.out_filename != "index.html")
        .map(|p| {
            let title = xml_escape(p.meta.title.as_deref().unwrap_or(&config.title));
            let url = xml_escape(&p.full_url);
            let desc = xml_escape(p.meta.description.as_deref().unwrap_or(&config.description));
            let date = p.meta.date.as_deref().map(format_rfc822).unwrap_or_default();
            let body_html = if p.is_md {
                let opts = Options::ENABLE_TABLES
                    | Options::ENABLE_STRIKETHROUGH
                    | Options::ENABLE_TASKLISTS;
                let parser = Parser::new_ext(&p.body, opts);
                let mut out = String::new();
                cm_html::push_html(&mut out, parser);
                convert_callouts(&out)
            } else {
                p.body.clone()
            };
            let cdata = body_html.replace("]]>", "]]>]]><![CDATA[");
            format!(
                "<item>\
                <title>{title}</title>\
                <link>{url}</link>\
                <description>{desc}</description>\
                <pubDate>{date}</pubDate>\
                <guid isPermaLink=\"true\">{url}</guid>\
                <content:encoded><![CDATA[{cdata}]]></content:encoded>\
                </item>"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let last_build = pages
        .iter()
        .find_map(|p| p.meta.date.as_deref())
        .map(format_rfc822)
        .unwrap_or_default();
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
        <rss version=\"2.0\" xmlns:content=\"http://purl.org/rss/1.1/modules/content/\"><channel>\
        <title>{}</title><link>{base}</link><description>{}</description>\
        <language>en</language><lastBuildDate>{last_build}</lastBuildDate>\
        {items}\
        </channel></rss>",
        xml_escape(&config.title),
        xml_escape(&config.description),
    )
}
