use crate::utils::html_escape;
use dialoguer::Input;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const LOGO: &[u8] = include_bytes!("./static/logo.png");
const STYLES: &[u8] = include_bytes!("./static/styles.css");

fn get_project_name() -> Result<Option<String>, std::io::Error> {
    let current_dir = std::env::current_dir()?
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string();

    let name: String = Input::new()
        .with_prompt(format!(
            "Project name (leave blank to initialise in the current directory '{current_dir}')"
        ))
        .allow_empty(true)
        .interact_text()?;

    if name.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(sanitise_string(&name)))
    }
}

fn prompt(label: &str) -> Result<String, std::io::Error> {
    Ok(Input::new().with_prompt(label).interact_text()?)
}

fn toml_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn sanitise_string(page_name: &str) -> String {
    page_name
        .trim()
        .to_lowercase()
        .replace(" ", "_")
        .to_string()
}

fn generate_navbar_list(navbar_items: &str) -> String {
    navbar_items
        .split(',')
        .map(|item| item.trim())
        .map(|item| {
            format!(
                "<li class=\"navbar__link\"><a href=\"/{}\">{}</a></li>",
                sanitise_string(item),
                item
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn new_page() -> Result<(), std::io::Error> {
    let filename = sanitise_string(&prompt("Filename (without extension)")?);
    let ext: String = Input::new()
        .with_prompt("File type")
        .default("md".to_string())
        .interact_text()?;
    let title = prompt("Title")?;
    let description = prompt("Description")?;
    let keywords = prompt("Keywords (comma-separated)")?;

    let ext = if ext == "html" { "html" } else { "md" };
    let path = format!("{filename}.{ext}");

    if fs::metadata(&path).is_ok() {
        println!("[WARNING] '{path}' already exists");
        return Err(std::io::Error::other("Operation canceled."));
    }

    let content = match ext {
        "html" => format!(
            "<!--\ntitle: {title}\ndescription: {description}\nkeywords: {keywords}\n-->\n\n<h1>{}</h1>\n",
            html_escape(&title)
        ),
        _ => format!(
            "---\ntitle: {title}\ndescription: {description}\nkeywords: {keywords}\n---\n\n# {title}\n"
        ),
    };

    fs::write(&path, content)?;
    println!("Created {path}");
    Ok(())
}

pub fn init_project() -> Result<(), std::io::Error> {
    let base = match get_project_name()? {
        None => ".".to_string(),
        Some(name) => {
            if fs::metadata(&name).is_ok() {
                println!("[WARNING] directory '{name}' already exists");
                return Err(std::io::Error::other("Operation canceled."));
            }
            fs::create_dir_all(&name)?;
            name
        }
    };

    let static_dir = format!("{base}/static");
    let templates_dir = format!("{base}/templates");
    let pages_dir = format!("{base}/pages");

    fs::create_dir_all(&static_dir)?;
    fs::create_dir_all(&templates_dir)?;
    fs::create_dir_all(&pages_dir)?;

    fs::write(format!("{static_dir}/logo.png"), LOGO)?;
    fs::write(format!("{static_dir}/styles.css"), STYLES)?;

    let title = prompt("Enter the title of your website")?;
    let base_url = prompt("Enter the base URL of your website (e.g. https://example.com)")?;
    let author = prompt("Enter the author name")?;
    let description = prompt("Enter the site description")?;
    let keywords = prompt("Enter keywords (comma-separated)")?;
    let pages = prompt("Enter navbar items (comma-separated)")?;

    let config = format!(
        "title = \"{}\"\nbase_url = \"{}\"\nauthor = \"{}\"\ndescription = \"{}\"\nkeywords = \"{}\"\n",
        toml_escape(&title),
        toml_escape(&base_url),
        toml_escape(&author),
        toml_escape(&description),
        toml_escape(&keywords),
    );
    fs::write(format!("{base}/rssg.toml"), config)?;

    let mut home = File::create(format!("{pages_dir}/index.md"))?;
    home.write_all(b"# Add your home page content here")?;

    for page in pages.split(',') {
        let page_dir = format!("{base}/pages/{}", sanitise_string(page));
        fs::create_dir_all(&page_dir)?;
        let mut f = File::create(format!("{page_dir}/index.md"))?;
        f.write_all(b"# Add your content here")?;
    }

    let navbar_list = generate_navbar_list(&pages);
    let template = "<!DOCTYPE html>
        <html lang=\"en\">
        <head>
            <meta http-equiv=\"content-type\" content=\"text/html; charset=utf-8\" />
            <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />
            <meta http-equiv=\"X-UA-Compatible\" content=\"ie=edge\" />
            <meta http-equiv=\"cache-control\" content=\"no-cache, must-revalidate, post-check=0, pre-check=0\" />
            <meta http-equiv=\"cache-control\" content=\"max-age=0\" />
            <meta http-equiv=\"expires\" content=\"0\" />
            <meta http-equiv=\"expires\" content=\"Tue, 01 Jan 1980 1:00:00 GMT\" />
            <meta http-equiv=\"pragma\" content=\"no-cache\" />
            <link rel=\"stylesheet\" href=\"/static/styles.css?v={{refresh_cache}}\" />
            <link rel=\"shortcut icon\" type=\"image/x-icon\" href=\"/static/logo.png?v={{refresh_cache}}\" />
            <title>{{title}}</title>
            <meta name=\"description\" content=\"{{description}}\" />
            <meta name=\"author\" content=\"{{author}}\" />
            <meta name=\"keywords\" content=\"{{keywords}}\" />
            <meta property=\"og:title\" content=\"{{title}}\" />
            <meta property=\"og:description\" content=\"{{description}}\" />
            <meta property=\"og:url\" content=\"{{page_url}}\" />
            <meta property=\"og:type\" content=\"website\" />
            <meta name=\"twitter:card\" content=\"summary\" />
            <meta name=\"twitter:title\" content=\"{{title}}\" />
            <meta name=\"twitter:description\" content=\"{{description}}\" />
            <link rel=\"canonical\" href=\"{{page_url}}\" />
        </head>
        <body>
            <input id=\"theme\" type=\"checkbox\" />
            <div class=\"scheme-wrapper\">
                <div class=\"container\">
                    <header>
                        <nav class=\"navbar\">
                            <a class=\"navbar__left\" href=\"/\">
                                <img
                                    class=\"navbar__logo\"
                                    title=\"rssg\"
                                    src=\"/static/logo.png\"
                                />
                            </a>
                            <ul class=\"navbar__right\">
                                {{navbar_list}}
                            </ul>
                        </nav>
                    </header>
                    <main></main>
                    <footer class=\"footer\">
                        <p>&copy; {{year}} {{author}}. All Rights Reserved.</p>
                    </footer>
                </div>
            </div>
        </body>
        </html>"
        .replace("{{navbar_list}}", &navbar_list);

    fs::write(Path::new(&templates_dir).join("template.html"), template)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitise_string() {
        assert_eq!(sanitise_string("My Page "), "my_page");
        assert_eq!(sanitise_string("  Another Page"), "another_page");
        assert_eq!(sanitise_string("Already_good"), "already_good");
    }

    #[test]
    fn test_generate_navbar_list() {
        let input = "Home, About, Contact";
        let expected = "<li class=\"navbar__link\"><a href=\"/home\">Home</a></li>\n\
                        <li class=\"navbar__link\"><a href=\"/about\">About</a></li>\n\
                        <li class=\"navbar__link\"><a href=\"/contact\">Contact</a></li>";
        assert_eq!(generate_navbar_list(input), expected);
    }
}
