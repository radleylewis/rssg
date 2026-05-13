<div align="center">
 <img src="https://github.com/radleylewis/rssg/assets/40852773/5bef91b6-10f3-425e-b3d7-52414faca447" width="250px">
 <p>A Static Site Generator built in Rust.</p>
</div>

## Introduction

rssg is not a replacement for [Hugo](https://gohugo.io) or [Zola](https://www.getzola.org). Those tools have their use case. If you need a templating engine, a theme ecosystem, or a large community, use them.

rssg is for anyone who wants to own their HTML and CSS completely - no templating language to learn, no theme to override, no abstraction between you and your output. You write one HTML file and one CSS file. rssg handles the rest.

**What rssg does:**

- Compiles to a single binary with no runtime dependencies
- Builds pages in parallel - fast regardless of site size
- Converts Markdown and HTML pages to a static site
- Generates post lists, pagination, tag pages, related articles, and section indexes automatically
- Produces an RSS feed, sitemap, and `robots.txt` with no configuration
- Injects structured data (JSON-LD) for WebSite, BlogPosting, and BreadcrumbList schemas on every page
- Outputs full Open Graph and Twitter card meta tags, canonical URLs, and `article:published_time` on dated posts
- Converts raster images to WebP at build time, rewrites all references, and serves the correct MIME types in dev
- Highlights code blocks at build time with no client-side JavaScript
- Renders Obsidian-style callout blocks (`[!NOTE]`, `[!WARNING]`, etc.) from Markdown blockquotes
- Displays estimated reading time and location on dated posts
- Ships a CSS-only dark/light theme toggle
- Uses no JavaScript frameworks, no tracking, no external dependencies in the browser - only a clipboard API call for code block copying and a fetch call for live reload in dev

**What rssg does not do:**

- Templating logic (conditionals, loops, partials) - your template is plain HTML with `{{placeholders}}`
- Themes or plugins
- Asset pipelines (no Sass, no bundling)

If you are comfortable writing HTML and CSS and want a fast, transparent build with strong SEO defaults out of the box, rssg is designed for you.

## Usage

Compile the binary:

```bash
cargo build --release
```

### Initialise a project

```bash
rssg init
```

Complete the prompts to scaffold a new project. Leave the project name blank to initialise in the current directory. This creates an `rssg.toml` config file, a `templates/` directory, and a `pages/` directory.

### Create a page

From inside your project directory, navigate to the directory where you want the page and run:

```bash
rssg new
```

You will be prompted for a filename, file type (`md` or `html`), title, description, and keywords. The file is created with front matter pre-filled.

**Markdown front matter:**
```markdown
---
title: Why the Euro makes no sense
description: An economic takedown of the most overengineered currency
tags: economics, money
date: 2027-01-15
location: Beijing, China
draft: true
---

# My Article
```

**HTML front matter:**
```html
<!--
title: Why the Euro makes no sense
description: An economic takedown of the most overengineered currency
tags: economics, money
date: 2027-01-15
location: Beijing, China
draft: true
-->

<h1>My Page</h1>
```

**Front matter fields:**

| Field | Description |
|---|---|
| `title` | Page title (overrides site title in `<title>`, OG tags, and RSS) |
| `description` | Page description (overrides site default in meta and RSS) |
| `tags` | Comma-separated tags (used for filtering, SEO keywords, and tag pages) |
| `date` | Publication date in `YYYY-MM-DD` format (shown above content, used to sort posts, included in RSS) |
| `last_edited` | Last edited date in `YYYY-MM-DD` format (shown in post header if different from `date`, used as sitemap `<lastmod>`) |
| `location` | Where the post was written (displayed in the post header) |
| `language` | BCP 47 language code for the page (sets `lang` on `<html>`, defaults to `en`) |
| `og_image` | Override the default OG image for this page |
| `draft` | Set to `true` to exclude the page from build output |

> **Tip:** For long descriptions, edit `rssg.toml` directly rather than typing in the prompt.

### Build

```bash
rssg build
```

Outputs to `dist/`. Also generates `sitemap.xml`, `robots.txt`, `404.html`, and `feed.xml`.

### Serve locally

```bash
rssg serve
rssg serve -p 3000
```

Serves the `dist/` directory at `http://localhost:8080` (default). Watches for file changes and rebuilds automatically - the browser reloads when a rebuild completes.

## Configuration

`rssg.toml` in your project root holds site-wide defaults. Per-page front matter overrides these where applicable.

```toml
title = "My Site"
base_url = "https://example.com"
author = "Your Name"
description = "Site description"
locale = "en_US"
posts_per_page = 10

# Image optimisation (optional)
optimize_images = true
max_image_width = 1200
webp_quality = 80
og_image = "/static/preview.jpg"
```

| Field | Default | Description |
|---|---|---|
| `title` | required | Site title used in `<title>`, OG tags, RSS, and JSON-LD |
| `base_url` | required | Canonical base URL (no trailing slash) |
| `author` | required | Author name used in footer, meta tags, and JSON-LD |
| `description` | required | Site-wide default description for meta and RSS |
| `locale` | `en_US` | OG locale tag (e.g. `en_GB`, `fr_FR`) |
| `posts_per_page` | `10` | Number of posts per paginated listing page |
| `optimize_images` | `false` | Convert raster images in `static/` to WebP during build |
| `max_image_width` | none | Resize images wider than this (pixels) before converting |
| `webp_quality` | `80` | WebP encoding quality (0-100) |
| `og_image` | none | Default OG image for pages without one (relative path or absolute URL) |

## Post lists

Any `index.md` or `index.html` inside a section directory (e.g. `pages/writing/`) will automatically have a list of sibling pages appended to it, sorted by date descending. Pages without a date appear at the end.

## RSS feed

An RSS feed is generated at `dist/feed.xml` and linked automatically in every page's `<head>`. It includes all non-index pages that have a `date` field in their front matter, sorted by date descending.

## Author

Radley E. Sidwell-Lewis
