<div align="center">
 <img src="https://github.com/radleylewis/rssg/assets/40852773/5bef91b6-10f3-425e-b3d7-52414faca447" width="250px">
 <p>A Static Site Generator built in Rust.</p>
</div>

## Introduction

Write your site in Markdown or HTML. rssg includes:

- Dark/light theme toggle (CSS only, no JavaScript)
- Code block language labels and copy button (minimal JavaScript, clipboard API only)
- Support for Markdown and HTML pages
- Front matter for per-page SEO metadata
- Build-time syntax highlighting for code blocks
- Auto-generated post lists for section index pages
- RSS feed generation
- Sitemap and robots.txt generation
- Local dev server with hot reload

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
description: An economic takedown of the most pantsuit wearing currency
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
description: An economic takedown of the most pantsuit wearing currency
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

`rssg.toml` in your project root holds site-wide defaults. Per-page front matter overrides these values.

```toml
title = "My Site"
base_url = "https://example.com"
author = "Your Name"
description = "Site description"
posts_per_page = 10
```

## Post lists

Any `index.md` or `index.html` inside a section directory (e.g. `pages/writing/`) will automatically have a list of sibling pages appended to it, sorted by date descending. Pages without a date appear at the end.

## RSS feed

An RSS feed is generated at `dist/feed.xml` and linked automatically in every page's `<head>`. It includes all non-index pages that have a `date` field in their front matter, sorted by date descending.

## Author

Radley E. Sidwell-Lewis
