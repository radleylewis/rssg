use crate::{
    frontmatter::{PageInfo, SiteConfig, parse_front_matter},
    highlight::{convert_callouts, highlight_code_blocks, syntax_highlight_css, CODE_COPY_SCRIPT},
    images::{copy_static_optimized, rewrite_image_refs},
    render::{
        apply_article_meta, generate_pagination_nav, generate_post_list, generate_rss,
        generate_sitemap, generate_tag_nav, render_page, section_url_for_pages,
    },
    utils::{
        build_timestamp, copy_directory, current_year, estimate_read_time, format_date,
        html_escape, path_to_url, read_all_files_recursive, slugify,
    },
};
use pulldown_cmark::{html, Options, Parser};
use std::{collections::HashMap, fs, path::{Path, PathBuf}};
use syntect::highlighting::ThemeSet;

fn convert_md_to_html(md_content: &str) -> String {
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(md_content, opts);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

pub fn build_project() -> Result<(), Box<dyn std::error::Error>> {
    let config: SiteConfig = toml::from_str(&fs::read_to_string("rssg.toml")?)?;
    let base = config.base_url.trim().trim_end_matches('/');
    let posts_per_page = config.posts_per_page.max(1);

    if fs::metadata("dist").is_ok() {
        fs::remove_dir_all("dist")?;
    }
    fs::create_dir_all("dist/static")?;
    let mut converted_images: Vec<String> = Vec::new();
    if Path::new("static").is_dir() {
        if config.optimize_images {
            copy_static_optimized(
                Path::new("static"),
                Path::new("dist/static"),
                config.max_image_width,
                config.webp_quality,
                &mut converted_images,
                "",
            )?;
        } else {
            copy_directory(Path::new("./static"), Path::new("./dist/static"))?;
        }
    }

    let ss = two_face::syntax::extra_newlines();
    let ts = ThemeSet::load_defaults();
    let highlight_css = syntax_highlight_css(&ts);

    let base_template = {
        let t = fs::read_to_string("templates/template.html")?
            .replace("{{refresh_cache}}", &build_timestamp().to_string())
            .replace("{{year}}", &current_year().to_string())
            .replace("{{author}}", &html_escape(&config.author))
            .replace("{{base_url}}", base)
            .replace("</head>", &format!("{highlight_css}</head>"))
            .replace("</body>", &format!("{CODE_COPY_SCRIPT}</body>"));
        if converted_images.is_empty() { t } else { rewrite_image_refs(&t, &converted_images) }
    };

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

    let mut dir_index: HashMap<PathBuf, Vec<usize>> = HashMap::new();
    for (i, page) in pages.iter().enumerate() {
        dir_index.entry(page.relative_dir.clone()).or_default().push(i);
    }

    let mut tag_map: HashMap<String, Vec<usize>> = HashMap::new();
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

    let render = |tmpl: &str, title: &str, desc: &str, kw: &str, url: &str, og_img: &str, name: &str, content: &str, lang: &str| -> String {
        let html = render_page(tmpl, title, desc, kw, url, og_img, name, content, lang);
        if converted_images.is_empty() { html } else { rewrite_image_refs(&html, &converted_images) }
    };

    for i in 0..pages.len() {
        let title = html_escape(pages[i].meta.title.as_deref().unwrap_or(&config.title));
        let description = html_escape(
            pages[i].meta.description.as_deref().unwrap_or(&config.description),
        );
        let keywords = html_escape(&pages[i].meta.tags.join(", "));

        let mut content = if pages[i].is_md {
            let html = convert_md_to_html(&pages[i].body);
            let html = highlight_code_blocks(&html, &ss);
            convert_callouts(&html)
        } else {
            highlight_code_blocks(&pages[i].body, &ss)
        };

        let location = pages[i].meta.location.as_deref()
            .map(|l| format!("<span class=\"post-location\">{l}</span>"))
            .unwrap_or_default();

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

        let current_page_name = pages[i].relative_dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let lang = pages[i].meta.language.as_deref().unwrap_or("en");
        let og_image = pages[i].meta.og_image.as_deref()
            .or(config.og_image.as_deref())
            .map(|p| if p.starts_with("http") { p.to_string() } else { format!("{base}{p}") })
            .unwrap_or_default();

        if pages[i].out_filename == "index.html" {
            let siblings: Vec<&PageInfo> = dir_index
                .get(&pages[i].relative_dir)
                .map(|indices| {
                    indices.iter()
                        .filter(|&&j| j != i && pages[j].out_filename != "index.html")
                        .map(|&j| &pages[j])
                        .collect()
                })
                .unwrap_or_default();

            let total_pages = if siblings.is_empty() {
                1
            } else {
                siblings.len().div_ceil(posts_per_page)
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
                render(&base_template, &title, &description, &keywords, &pages[i].full_url, &og_image, current_page_name, &page1_content, lang),
            )?;
            urls.push((pages[i].page_url.clone(), None));

            for page_num in 2..=total_pages {
                let chunk: Vec<&PageInfo> = siblings.iter().copied()
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
                    render(&base_template, &page_title, &description, &keywords, &full_url, &og_image, current_page_name, &page_content, lang),
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
            let rendered = render(&base_template, &title, &description, &keywords, &pages[i].full_url, &og_image, current_page_name, &content, lang);
            let rendered = if let Some(published) = pages[i].meta.date.as_deref() {
                apply_article_meta(&rendered, published, pages[i].meta.last_edited.as_deref(), &config.author)
            } else {
                rendered
            };
            fs::write(format!("{out_dir}/index.html"), rendered)?;
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
        let total_tag_pages = tagged_pages.len().div_ceil(posts_per_page);

        for page_num in 1..=total_tag_pages {
            let chunk: Vec<&PageInfo> = tagged_pages.iter().copied()
                .skip((page_num - 1) * posts_per_page)
                .take(posts_per_page)
                .collect();
            let page_tag_url = if page_num == 1 { tag_url.clone() } else { format!("{tag_url}{page_num}/") };
            let page_full_url = format!("{base}{page_tag_url}");
            let page_title = if page_num == 1 { tag_title.clone() } else { format!("{tag_title} — Page {page_num}") };
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
                render(&base_template, &page_title, &config.description, "", &page_full_url, "", "", &content, "en"),
            )?;
            if page_num == 1 {
                urls.push((page_tag_url, None));
            }
        }
    }

    fs::write("dist/sitemap.xml", generate_sitemap(&config.base_url, &urls))?;
    fs::write("dist/feed.xml", generate_rss(&config, &pages))?;

    let not_found_template = base_template.lines()
        .filter(|l| !l.contains("og:url") && !l.contains("canonical"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(
        "dist/404.html",
        render(&not_found_template, "404 - Page Not Found", "The page you are looking for does not exist.", "", "", "", "", "<h1>404</h1><p>Page not found.</p>", "en"),
    )?;

    fs::write(
        "dist/robots.txt",
        format!("User-agent: *\nAllow: /\nSitemap: {base}/sitemap.xml\n"),
    )?;

    Ok(())
}
