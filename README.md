<div align="center">
 <img src="https://github.com/radleylewis/rssg/assets/40852773/5bef91b6-10f3-425e-b3d7-52414faca447" width="250px">
 <p>A Static Site Generator built in Rust.</p>
</div>

## Introduction

Write your site in Markdown or HTML. rssg includes:

- Dark/light theme toggle — zero JavaScript
- Support for Markdown and HTML pages
- Front matter for per-page SEO metadata
- Build-time syntax highlighting for code blocks
- Sitemap and robots.txt generation
- Local dev server

## Usage

Compile the binary:

```bash
cargo build --release
```

### Initialise a project

```bash
rssg init
```

Complete the prompts to scaffold a new project. This creates an `rssg.toml` config file, a `templates/` directory, and a `pages/` directory.

### Create a page

From inside your project directory, navigate to the directory where you want the page and run:

```bash
rssg new
```

You will be prompted for a filename, file type (`md` or `html`), title, description, and keywords. The file is created with front matter pre-filled.

**Markdown front matter:**
```markdown
---
title: My Article
description: A short summary
keywords: rust, web
---

# My Article
```

**HTML front matter:**
```html
<!--
title: My Page
description: A short summary
keywords: rust, web
-->

<h1>My Page</h1>
```

### Build

```bash
rssg build
```

Outputs to `dist/`. Also generates `sitemap.xml`, `robots.txt`, and `404.html`.

### Serve locally

```bash
rssg serve
rssg serve -p 3000
```

Serves the `dist/` directory at `http://localhost:8080` (default).

## Configuration

`rssg.toml` in your project root:

```toml
title = "My Site"
base_url = "https://example.com"
author = "Your Name"
description = "Site description"
keywords = "keyword1, keyword2"
```

## Author

Radley E. Sidwell-Lewis
