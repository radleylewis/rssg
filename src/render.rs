use crate::frontmatter::{PageInfo, SiteConfig};
use crate::utils::{format_date, format_rfc822, html_escape, slugify, xml_escape};

pub fn apply_article_meta(html: &str, published: &str, modified: Option<&str>, author: &str) -> String {
    let article_tags = format!(
        "<meta property=\"og:type\" content=\"article\" />\n\
        <meta property=\"article:published_time\" content=\"{published}\" />\n\
        {modified_tag}\
        <meta property=\"article:author\" content=\"{author}\" />",
        modified_tag = modified
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
        .filter(|p| {
            p.out_filename != "index.html"
                && p.page_url != current.page_url
                && p.meta.tags.iter().any(|t| current.meta.tags.contains(t))
        })
        .map(|p| {
            let shared = p.meta.tags.iter().filter(|t| current.meta.tags.contains(t)).count();
            (shared, p)
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
                .map(|d| format!(" - <span class=\"related__date\">({})</span>", format_date(d)))
                .unwrap_or_default();
            format!("<li><a href=\"{}\">{title}</a>{date}</li>", p.page_url)
        })
        .collect();
    format!("<hr class=\"divider\" /><h3>Related Articles</h3><ul>{items}</ul>")
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
            format!(
                "<item><title>{title}</title><link>{url}</link><description>{desc}</description><pubDate>{date}</pubDate><guid isPermaLink=\"true\">{url}</guid></item>"
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
        <rss version=\"2.0\"><channel>\
        <title>{}</title><link>{base}</link><description>{}</description>\
        <language>en</language><lastBuildDate>{last_build}</lastBuildDate>\
        {items}\
        </channel></rss>",
        xml_escape(&config.title),
        xml_escape(&config.description),
    )
}
