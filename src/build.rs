use crate::utils::html_escape;
use pulldown_cmark::{html, Parser};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::{self, DirEntry},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use syntect::{
    highlighting::ThemeSet,
    html::{css_for_theme_with_class_style, ClassStyle, ClassedHTMLGenerator},
    util::LinesWithEndings,
};

fn default_posts_per_page() -> usize {
    10
}

#[derive(Deserialize)]
struct SiteConfig {
    title: String,
    base_url: String,
    author: String,
    description: String,
    #[serde(default = "default_posts_per_page")]
    posts_per_page: usize,
}

#[derive(Default)]
struct PageMeta {
    title: Option<String>,
    description: Option<String>,
    date: Option<String>,
    last_edited: Option<String>,
    tags: Vec<String>,
    location: Option<String>,
    language: Option<String>,
    draft: bool,
}

struct PageInfo {
    relative_dir: PathBuf,
    out_filename: String,
    page_url: String,
    full_url: String,
    meta: PageMeta,
    body: String,
    is_md: bool,
}

fn html_decode(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn extract_language(opening_tag: &str) -> Option<&str> {
    let start = opening_tag.find("language-")? + "language-".len();
    let rest = &opening_tag[start..];
    let end = rest.find(['"', '\''])?;
    Some(&rest[..end])
}

fn highlight_code_blocks(html: &str, ss: &syntect::parsing::SyntaxSet) -> String {
    let mut result = String::with_capacity(html.len() + 1024);
    let mut remaining = html;

    while let Some(start) = remaining.find("<pre><code") {
        result.push_str(&remaining[..start]);
        remaining = &remaining[start..];

        // Skip past <pre> (5 chars) before searching for '>' to find end of <code...> tag
        let tag_end = match remaining[5..].find('>') {
            Some(i) => 5 + i + 1,
            None => break,
        };

        let lang = extract_language(&remaining[..tag_end]);
        let close = "</code></pre>";

        match remaining.find(close) {
            Some(end) => {
                let code_html = &remaining[tag_end..end];
                let code = html_decode(code_html);

                let highlighted = lang
                    .and_then(|l| ss.find_syntax_by_token(l))
                    .map(|syntax| {
                        let mut gen = ClassedHTMLGenerator::new_with_class_style(
                            syntax,
                            ss,
                            ClassStyle::Spaced,
                        );
                        for line in LinesWithEndings::from(&code) {
                            let _ = gen.parse_html_for_line_which_includes_newline(line);
                        }
                        gen.finalize()
                    })
                    .unwrap_or_else(|| code_html.to_string());

                let lang_label = lang.unwrap_or("");
                result.push_str(&format!(
                    "<div class=\"code-block\">\
                    <div class=\"code-block__header\">\
                    <span class=\"code-block__lang\">{lang_label}</span>\
                    <button class=\"code-block__copy\">Copy</button>\
                    </div>\
                    <pre><code>{highlighted}</code></pre>\
                    </div>"
                ));
                remaining = &remaining[end + close.len()..];
            }
            None => break,
        }
    }

    result.push_str(remaining);
    result
}

fn syntax_highlight_css(ts: &ThemeSet) -> String {
    let light = ts
        .themes
        .get("InspiredGitHub")
        .and_then(|t| css_for_theme_with_class_style(t, ClassStyle::Spaced).ok())
        .unwrap_or_default();
    let dark = ts
        .themes
        .get("base16-ocean.dark")
        .and_then(|t| css_for_theme_with_class_style(t, ClassStyle::Spaced).ok())
        .unwrap_or_default();
    format!(
        "<style>\
        pre code{{background:none;padding:0;}}\
        .code-block{{margin:1.5rem 0;}}\
        .code-block pre{{margin:0;border-radius:0 0 4px 4px;}}\
        .code-block__header{{display:flex;justify-content:space-between;align-items:center;\
          background:#e8e8e8;padding:0.3rem 0.75rem;border-radius:4px 4px 0 0;font-size:0.8rem;}}\
        .code-block__lang{{color:#666;text-transform:uppercase;font-size:0.75rem;letter-spacing:0.05em;}}\
        .code-block__copy{{all:unset;cursor:pointer;color:#666;padding:0.15rem 0.5rem;\
          border:1px solid #aaa;border-radius:3px;font-size:0.75rem;}}\
        .code-block__copy:hover{{color:var(--button-bg,#b5a642);border-color:var(--button-bg,#b5a642);}}\
        {light}\
        @media(prefers-color-scheme:dark){{\
          {dark}\
          .code-block__header{{background:#2d3035;}}\
          .code-block__lang,.code-block__copy{{color:#ccc;}}\
        }}\
        #theme:checked ~ * .code-block__header{{background:#2d3035;}}\
        #theme:checked ~ * .code-block__lang,\
        #theme:checked ~ * .code-block__copy{{color:#ccc;}}\
        @media(prefers-color-scheme:dark){{\
          #theme:checked ~ * .code-block__header{{background:#e8e8e8;}}\
          #theme:checked ~ * .code-block__lang,\
          #theme:checked ~ * .code-block__copy{{color:#666;}}\
        }}\
        </style>"
    )
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

fn parse_front_matter(content: &str) -> (PageMeta, String) {
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

fn convert_md_to_html(md_content: &str) -> String {
    let parser = Parser::new(md_content);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

fn copy_directory(src: &Path, dest: &Path) -> Result<(), std::io::Error> {
    if !src.is_dir() {
        return Err(std::io::Error::other("Source is not a directory"));
    }
    if !dest.is_dir() {
        return Err(std::io::Error::other("Destination is not a directory"));
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let Some(name) = path.file_name() else { continue };
        let dest_path = dest.join(name);
        if path.is_dir() {
            fs::create_dir_all(&dest_path)?;
            copy_directory(&path, &dest_path)?;
        } else if path.is_file() {
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

fn read_all_files_recursive(
    dir: &Path,
    files: &mut Vec<DirEntry>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            read_all_files_recursive(&path, files)?;
        } else if path.is_file() {
            files.push(entry);
        }
    }
    Ok(())
}

fn build_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

fn current_year() -> u32 {
    1970 + (build_timestamp() / 31_557_600) as u32
}

fn path_to_url(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn estimate_read_time(body: &str) -> usize {
    let words = body.split_whitespace().count();
    ((words as f64 / 200.0).ceil() as usize).max(1)
}

fn format_date(date: &str) -> String {
    const MONTHS: [&str; 12] = [
        "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December",
    ];
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 { return date.to_string(); }
    let month_idx: usize = parts[1].parse().unwrap_or(0);
    let day: u32 = parts[2].parse().unwrap_or(0);
    if month_idx == 0 || month_idx > 12 || day == 0 { return date.to_string(); }
    format!("{} {} {}", day, MONTHS[month_idx - 1], parts[0])
}

fn slugify(s: &str) -> String {
    s.trim().to_lowercase().replace(' ', "-")
}

fn format_rfc822(date: &str) -> String {
    const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun",
        "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 { return date.to_string(); }
    let Ok(y) = parts[0].parse::<i32>() else { return date.to_string() };
    let Ok(m) = parts[1].parse::<u32>() else { return date.to_string() };
    let Ok(d) = parts[2].parse::<u32>() else { return date.to_string() };
    if m == 0 || m > 12 || d == 0 || d > 31 { return date.to_string(); }
    // Tomohiko Sakamoto's algorithm for day-of-week
    let t: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let yr = if m < 3 { y - 1 } else { y };
    let dow = ((yr + yr/4 - yr/100 + yr/400 + t[(m-1) as usize] + d as i32).rem_euclid(7)) as usize;
    format!("{}, {:02} {} {} 00:00:00 +0000", DAYS[dow], d, MONTHS[(m-1) as usize], y)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn generate_tag_nav(all_tags: &[String], active_tag: Option<&str>, section_url: &str) -> String {
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
            "<a href=\"/tags/{slug}/\" class=\"{class}\">{}</a>", html_escape(tag)
        ));
    }
    format!("<nav class=\"tag-filters\">{html}</nav>")
}

fn section_url_for_pages(pages: &[&PageInfo]) -> String {
    for p in pages {
        if let Some(first) = p.relative_dir.components().next() {
            let dir = first.as_os_str().to_string_lossy();
            return format!("/{dir}/");
        }
    }
    "/".to_string()
}

fn generate_post_list(pages: &[&PageInfo]) -> String {
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

fn generate_pagination_nav(current: usize, total: usize, section_url: &str) -> String {
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

fn generate_sitemap(base_url: &str, urls: &[(String, Option<String>)]) -> String {
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

fn generate_rss(config: &SiteConfig, pages: &[PageInfo]) -> String {
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

fn add_active_id_to_navbar(html: &str, page_name: &str) -> String {
    let search = format!("href=\"/{}/\"", page_name);
    let replace = format!("href=\"/{}/\" id=\"active\"", page_name);
    html.replace(&search, &replace)
}

fn render_page(
    base_template: &str,
    title: &str,
    description: &str,
    keywords: &str,
    full_url: &str,
    current_page_name: &str,
    content: &str,
    lang: &str,
) -> String {
    let rendered = base_template
        .replace("{{title}}", title)
        .replace("{{description}}", description)
        .replace("{{keywords}}", keywords)
        .replace("{{page_url}}", full_url)
        .replace("{{lang}}", lang);
    let rendered = add_active_id_to_navbar(&rendered, current_page_name);
    rendered.replace(
        "<main></main>",
        &format!("<main><div class=\"content\">{content}</div></main>"),
    )
}

const CODE_COPY_SCRIPT: &str = "<script>\
document.querySelectorAll('.code-block__copy').forEach(function(btn){\
  btn.addEventListener('click',function(){\
    var code=btn.closest('.code-block').querySelector('code');\
    navigator.clipboard.writeText(code.innerText).then(function(){\
      btn.textContent='Copied!';\
      setTimeout(function(){btn.textContent='Copy';},2000);\
    });\
  });\
});\
</script>";

pub fn build_project() -> Result<(), Box<dyn std::error::Error>> {
    let config: SiteConfig = toml::from_str(&fs::read_to_string("rssg.toml")?)?;
    let base = config.base_url.trim().trim_end_matches('/');
    let posts_per_page = config.posts_per_page.max(1);

    if fs::metadata("dist").is_ok() {
        fs::remove_dir_all("dist")?;
    }
    fs::create_dir_all("dist/static")?;
    if Path::new("static").is_dir() {
        copy_directory(Path::new("./static"), Path::new("./dist/static"))?;
    }

    let ss = two_face::syntax::extra_newlines();
    let ts = ThemeSet::load_defaults();
    let highlight_css = syntax_highlight_css(&ts);

    let base_template = fs::read_to_string("templates/template.html")?
        .replace("{{refresh_cache}}", &build_timestamp().to_string())
        .replace("{{year}}", &current_year().to_string())
        .replace("{{author}}", &html_escape(&config.author))
        .replace("{{base_url}}", base)
        .replace("</head>", &format!("{highlight_css}</head>"))
        .replace("</body>", &format!("{CODE_COPY_SCRIPT}</body>"));

    // --- Pass 1: collect all page metadata ---
    let mut raw_files = Vec::new();
    read_all_files_recursive(Path::new("pages"), &mut raw_files)?;

    let mut pages: Vec<PageInfo> = Vec::new();

    for entry in raw_files {
        let page_path = entry.path();
        let extension = match page_path.extension() {
            Some(e) => e.to_owned(),
            None => continue,
        };

        let raw = fs::read_to_string(&page_path)?;
        let (meta, body) = parse_front_matter(&raw);

        if meta.draft {
            continue;
        }

        let relative_path = page_path.strip_prefix(Path::new("pages"))?;
        let relative_dir = match relative_path.parent() {
            Some(d) => d.to_path_buf(),
            None => continue,
        };
        let stem = match page_path.file_stem() {
            Some(s) => s.to_string_lossy().into_owned(),
            None => continue,
        };
        let out_filename = format!("{stem}.html");

        let page_url = if out_filename == "index.html" {
            if relative_dir.as_os_str().is_empty() {
                "/".to_string()
            } else {
                format!("/{}/", path_to_url(&relative_dir))
            }
        } else if relative_dir.as_os_str().is_empty() {
            format!("/{stem}/")
        } else {
            format!("/{}/{stem}/", path_to_url(&relative_dir))
        };

        let full_url = format!("{base}{page_url}");
        let is_md = extension == "md";

        pages.push(PageInfo {
            relative_dir,
            out_filename,
            page_url,
            full_url,
            meta,
            body,
            is_md,
        });
    }

    pages.sort_by(|a, b| b.meta.date.cmp(&a.meta.date));

    // Group page indices by directory
    let mut dir_index: HashMap<PathBuf, Vec<usize>> = HashMap::new();
    for (i, page) in pages.iter().enumerate() {
        dir_index.entry(page.relative_dir.clone()).or_default().push(i);
    }

    // Collect tag → page index mappings (index pages are section landing pages, not posts)
    let mut tag_map: HashMap<String, Vec<usize>> = HashMap::new();
    // All unique tags per top-level section (for tag page navs)
    let mut section_all_tags: HashMap<String, Vec<String>> = HashMap::new();
    for (i, page) in pages.iter().enumerate() {
        if page.out_filename == "index.html" {
            continue;
        }
        for tag in &page.meta.tags {
            tag_map.entry(slugify(tag)).or_default().push(i);
        }
        let section = page.relative_dir.components().next()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .unwrap_or_default();
        let entry = section_all_tags.entry(section).or_default();
        for tag in &page.meta.tags {
            if !entry.contains(tag) {
                entry.push(tag.clone());
            }
        }
    }
    for tags in section_all_tags.values_mut() {
        tags.sort();
    }

    // --- Pass 2: render pages ---
    let mut urls: Vec<(String, Option<String>)> = Vec::new();

    for i in 0..pages.len() {
        let title = html_escape(pages[i].meta.title.as_deref().unwrap_or(&config.title));
        let description = html_escape(
            pages[i]
                .meta
                .description
                .as_deref()
                .unwrap_or(&config.description),
        );
        let keywords = html_escape(&pages[i].meta.tags.join(", "));

        let mut content = if pages[i].is_md {
            highlight_code_blocks(&convert_md_to_html(&pages[i].body), &ss)
        } else {
            highlight_code_blocks(&pages[i].body, &ss)
        };

        let location = pages[i].meta.location.as_deref().map(|l| {
            format!("<span class=\"post-location\">{l}</span>")
        }).unwrap_or_default();

        if pages[i].meta.date.is_some() || !location.is_empty() {
            let date_html = pages[i].meta.date.as_deref().map(|d| {
                format!("<time class=\"post-date\" datetime=\"{d}\">{}</time>", format_date(d))
            }).unwrap_or_default();
            let edited_html = pages[i].meta.last_edited.as_deref()
                .filter(|&e| Some(e) != pages[i].meta.date.as_deref())
                .map(|e| format!("<time class=\"post-updated\" datetime=\"{e}\">Updated: {}</time>", format_date(e)))
                .unwrap_or_default();
            let dates_html = if edited_html.is_empty() {
                date_html
            } else {
                format!("<div class=\"post-dates\">{date_html}{edited_html}</div>")
            };
            let read_time = format!("<span class=\"post-read-time\">{} min read</span>", estimate_read_time(&pages[i].body));
            content = format!("<div class=\"post-header\">{dates_html}{read_time}{location}</div>\n{content}");
        }

        let current_page_name = pages[i]
            .relative_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let lang = pages[i].meta.language.as_deref().unwrap_or("en");

        if pages[i].out_filename == "index.html" {
            let siblings: Vec<&PageInfo> = dir_index
                .get(&pages[i].relative_dir)
                .map(|indices| {
                    indices
                        .iter()
                        .filter(|&&j| j != i && pages[j].out_filename != "index.html")
                        .map(|&j| &pages[j])
                        .collect()
                })
                .unwrap_or_default();

            let total_pages = if siblings.is_empty() {
                1
            } else {
                (siblings.len() + posts_per_page - 1) / posts_per_page
            };

            let section_url = &pages[i].page_url;
            let section_tags: Vec<String> = {
                let mut set = std::collections::BTreeSet::new();
                for p in &siblings {
                    for tag in &p.meta.tags {
                        set.insert(tag.clone());
                    }
                }
                set.into_iter().collect()
            };
            let tag_nav = generate_tag_nav(&section_tags, None, section_url);

            let chunk: Vec<&PageInfo> = siblings.iter().copied().take(posts_per_page).collect();
            let page1_content = format!(
                "{content}{tag_nav}{}{}",
                generate_post_list(&chunk),
                generate_pagination_nav(1, total_pages, section_url)
            );

            let out_dir = format!("dist/{}", path_to_url(&pages[i].relative_dir));
            fs::create_dir_all(&out_dir)?;
            fs::write(
                format!("{out_dir}/index.html"),
                render_page(
                    &base_template,
                    &title,
                    &description,
                    &keywords,
                    &pages[i].full_url,
                    current_page_name,
                    &page1_content,
                    lang,
                ),
            )?;
            urls.push((pages[i].page_url.clone(), None));

            for page_num in 2..=total_pages {
                let chunk: Vec<&PageInfo> = siblings
                    .iter()
                    .copied()
                    .skip((page_num - 1) * posts_per_page)
                    .take(posts_per_page)
                    .collect();

                let page_url = format!("{section_url}{page_num}/");
                let full_url = format!("{base}{page_url}");
                let page_title = format!("{title} — Page {page_num}");
                let page_content = format!(
                    "{tag_nav}{}{}",
                    generate_post_list(&chunk),
                    generate_pagination_nav(page_num, total_pages, section_url)
                );

                let page_out_dir = format!("dist/{}/{page_num}", path_to_url(&pages[i].relative_dir));
                fs::create_dir_all(&page_out_dir)?;
                fs::write(
                    format!("{page_out_dir}/index.html"),
                    render_page(
                        &base_template,
                        &page_title,
                        &description,
                        &keywords,
                        &full_url,
                        current_page_name,
                        &page_content,
                        lang,
                    ),
                )?;
            }
        } else {
            let stem = pages[i].out_filename.trim_end_matches(".html");
            let out_dir = if pages[i].relative_dir.as_os_str().is_empty() {
                format!("dist/{stem}")
            } else {
                format!("dist/{}/{stem}", path_to_url(&pages[i].relative_dir))
            };
            fs::create_dir_all(&out_dir)?;
            fs::write(
                format!("{out_dir}/index.html"),
                render_page(
                    &base_template,
                    &title,
                    &description,
                    &keywords,
                    &pages[i].full_url,
                    current_page_name,
                    &content,
                    lang,
                ),
            )?;
            let lastmod = pages[i].meta.last_edited.clone().or_else(|| pages[i].meta.date.clone());
            urls.push((pages[i].page_url.clone(), lastmod));
        }
    }

    // --- Tag pages ---
    fs::create_dir_all("dist/tags")?;
    for (tag_slug, indices) in &tag_map {
        let tagged_pages: Vec<&PageInfo> = indices.iter().map(|&i| &pages[i]).collect();
        let tag_url = format!("/tags/{tag_slug}/");
        let tag_title = html_escape(tag_slug);
        let section_url = section_url_for_pages(&tagged_pages);
        let section_key = tagged_pages.iter()
            .find_map(|p| p.relative_dir.components().next())
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .unwrap_or_default();
        let section_tags = section_all_tags.get(&section_key).cloned().unwrap_or_default();
        let tag_nav = generate_tag_nav(&section_tags, Some(tag_slug), &section_url);

        let total_tag_pages =
            (tagged_pages.len() + posts_per_page - 1) / posts_per_page;

        for page_num in 1..=total_tag_pages {
            let chunk: Vec<&PageInfo> = tagged_pages
                .iter()
                .copied()
                .skip((page_num - 1) * posts_per_page)
                .take(posts_per_page)
                .collect();

            let page_tag_url = if page_num == 1 {
                tag_url.clone()
            } else {
                format!("{tag_url}{page_num}/")
            };
            let page_full_url = format!("{base}{page_tag_url}");
            let page_title = if page_num == 1 {
                tag_title.clone()
            } else {
                format!("{tag_title} — Page {page_num}")
            };

            let content = format!(
                "{tag_nav}{}{}",
                generate_post_list(&chunk),
                generate_pagination_nav(page_num, total_tag_pages, &tag_url)
            );

            let out_dir = if page_num == 1 {
                format!("dist/tags/{tag_slug}")
            } else {
                format!("dist/tags/{tag_slug}/{page_num}")
            };
            fs::create_dir_all(&out_dir)?;
            fs::write(
                format!("{out_dir}/index.html"),
                render_page(
                    &base_template,
                    &page_title,
                    &config.description,
                    "",
                    &page_full_url,
                    "",
                    &content,
                    "en",
                ),
            )?;
            if page_num == 1 {
                urls.push((page_tag_url, None));
            }
        }
    }

    fs::write("dist/sitemap.xml", generate_sitemap(&config.base_url, &urls))?;
    fs::write("dist/feed.xml", generate_rss(&config, &pages))?;

    let not_found_template = base_template
        .lines()
        .filter(|l| !l.contains("og:url") && !l.contains("canonical"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(
        "dist/404.html",
        render_page(
            &not_found_template,
            "404 - Page Not Found",
            "The page you are looking for does not exist.",
            "",
            "",
            "",
            "<h1>404</h1><p>Page not found.</p>",
            "en",
        ),
    )?;

    fs::write(
        "dist/robots.txt",
        format!("User-agent: *\nAllow: /\nSitemap: {base}/sitemap.xml\n"),
    )?;

    Ok(())
}
