use crate::utils::html_escape;
use pulldown_cmark::{html, Parser};
use serde::Deserialize;
use std::{
    fs::{self, DirEntry},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use syntect::{
    highlighting::ThemeSet,
    html::{css_for_theme_with_class_style, ClassStyle, ClassedHTMLGenerator},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

#[derive(Deserialize)]
struct SiteConfig {
    title: String,
    base_url: String,
    author: String,
    description: String,
    keywords: String,
}

#[derive(Default)]
struct PageMeta {
    title: Option<String>,
    description: Option<String>,
    keywords: Option<String>,
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

fn highlight_code_blocks(html: &str, ss: &SyntaxSet) -> String {
    let mut result = String::with_capacity(html.len());
    let mut remaining = html;

    while let Some(start) = remaining.find("<pre><code") {
        result.push_str(&remaining[..start]);
        remaining = &remaining[start..];

        let tag_end = match remaining.find('>') {
            Some(i) => i + 1,
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
                        format!("<pre><code>{}</code></pre>", gen.finalize())
                    })
                    .unwrap_or_else(|| format!("<pre><code>{code_html}</code></pre>"));

                result.push_str(&highlighted);
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
        pre {{ border-radius: 4px; padding: 1rem; overflow-x: auto; }}\
        pre code {{ background: none; padding: 0; font-size: 0.875rem; }}\
        {light}\
        @media(prefers-color-scheme:dark){{ {dark} }}\
        </style>"
    )
}

fn parse_meta_lines(text: &str) -> PageMeta {
    let mut meta = PageMeta::default();
    for line in text.lines() {
        if let Some((key, value)) = line.split_once(':') {
            match key.trim() {
                "title" => meta.title = Some(value.trim().to_string()),
                "description" => meta.description = Some(value.trim().to_string()),
                "keywords" => meta.keywords = Some(value.trim().to_string()),
                _ => {}
            }
        }
    }
    meta
}

fn parse_front_matter(content: &str) -> (PageMeta, String) {
    // Markdown: --- delimiters
    if let Some(rest) = content.strip_prefix("---") {
        let rest = rest.trim_start_matches('\n');
        if let Some(end) = rest.find("\n---") {
            let meta = parse_meta_lines(&rest[..end]);
            let body = rest[end + 4..].trim_start_matches('\n').to_string();
            return (meta, body);
        }
        eprintln!("[WARNING] Front matter opening '---' found but closing '---' is missing");
    }

    // HTML: <!-- --> comment block
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

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn generate_sitemap(base_url: &str, urls: &[String]) -> String {
    let base = xml_escape(base_url.trim().trim_end_matches('/'));
    let entries = urls
        .iter()
        .map(|url| format!("<url><loc>{base}{}</loc></url>", xml_escape(url)))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n{entries}\n</urlset>"
    )
}

fn add_active_id_to_navbar(html: &str, page_name: &str) -> String {
    let search = format!("href=\"/{}\"", page_name);
    let replace = format!("href=\"/{}\" id=\"active\"", page_name);
    html.replace(&search, &replace)
}

pub fn build_project() -> Result<(), Box<dyn std::error::Error>> {
    let config: SiteConfig = toml::from_str(&fs::read_to_string("rssg.toml")?)?;
    let base = config.base_url.trim().trim_end_matches('/');

    if fs::metadata("dist").is_ok() {
        fs::remove_dir_all("dist")?;
    }
    fs::create_dir_all("dist/static")?;
    copy_directory(Path::new("./static"), Path::new("./dist/static"))?;

    let mut pages = Vec::new();
    read_all_files_recursive(Path::new("pages"), &mut pages)?;

    let mut urls: Vec<String> = Vec::new();

    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let highlight_css = syntax_highlight_css(&ts);

    let base_template = fs::read_to_string("templates/template.html")?
        .replace("{{refresh_cache}}", &build_timestamp().to_string())
        .replace("{{year}}", &current_year().to_string())
        .replace("{{author}}", &html_escape(&config.author))
        .replace("</head>", &format!("{highlight_css}</head>"));

    for page in pages {
        let page_path = page.path();
        let extension = match page_path.extension() {
            Some(e) => e.to_owned(),
            None => continue,
        };

        let raw = fs::read_to_string(&page_path)?;
        let (meta, body) = parse_front_matter(&raw);

        let title = html_escape(&meta.title.unwrap_or_else(|| config.title.clone()));
        let description = html_escape(&meta.description.unwrap_or_else(|| config.description.clone()));
        let keywords = html_escape(&meta.keywords.unwrap_or_else(|| config.keywords.clone()));

        let content = if extension == "md" {
            highlight_code_blocks(&convert_md_to_html(&body), &ss)
        } else {
            highlight_code_blocks(&body, &ss)
        };

        let current_page = page_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let relative_path = page_path.strip_prefix(Path::new("pages"))?;
        let relative_dir = match relative_path.parent() {
            Some(d) => d,
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
                format!("/{}/", path_to_url(relative_dir))
            }
        } else if relative_dir.as_os_str().is_empty() {
            format!("/{out_filename}")
        } else {
            format!("/{}/{out_filename}", path_to_url(relative_dir))
        };

        let full_url = format!("{base}{page_url}");

        let rendered = base_template.clone()
            .replace("{{title}}", &title)
            .replace("{{description}}", &description)
            .replace("{{keywords}}", &keywords)
            .replace("{{page_url}}", &full_url);

        let rendered = add_active_id_to_navbar(&rendered, current_page);
        let rendered = rendered.replace(
            "<main></main>",
            &format!("<main><div class=\"content\">{content}</div></main>"),
        );

        let out_dir = format!("dist/{}", path_to_url(relative_dir));
        fs::create_dir_all(&out_dir)?;
        fs::write(format!("{out_dir}/{out_filename}"), rendered)?;

        urls.push(page_url);
    }

    fs::write("dist/sitemap.xml", generate_sitemap(&config.base_url, &urls))?;

    let not_found = base_template
        .replace("{{title}}", "404 - Page Not Found")
        .replace("{{description}}", "The page you are looking for does not exist.")
        .replace("{{keywords}}", "")
        .replace("{{page_url}}", &format!("{base}/404.html"))
        .replace(
            "<main></main>",
            "<main><div class=\"content\"><h1>404</h1><p>Page not found.</p></div></main>",
        );
    fs::write("dist/404.html", not_found)?;

    fs::write(
        "dist/robots.txt",
        format!("User-agent: *\nAllow: /\nSitemap: {base}/sitemap.xml\n"),
    )?;

    Ok(())
}
