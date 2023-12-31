use dialoguer::Input;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const LOGO: &[u8] = include_bytes!("./static/logo.png");
const STYLES: &[u8] = include_bytes!("./static/styles.css");

fn get_timestamp() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    return since_the_epoch.as_secs();
}

fn get_project_name() -> Result<String, std::io::Error> {
    let default_project_name: String = "my-project".to_string();
    let title: String = Input::new()
        .with_prompt("What is the name of your new project?")
        .default(default_project_name)
        .interact_text()?;

    let sanitised_project_name = sanitise_string(&title);
    Ok(sanitised_project_name)
}

fn get_website_title() -> Result<String, std::io::Error> {
    let title: String = Input::new()
        .with_prompt("Enter the title of your new website")
        .interact_text()?;

    Ok(title)
}

fn get_website_author() -> Result<String, std::io::Error> {
    let author: String = Input::new()
        .with_prompt("Enter the author of your new website")
        .interact_text()?;

    Ok(author)
}

fn get_website_description() -> Result<String, std::io::Error> {
    let description: String = Input::new()
        .with_prompt("Enter the description of your new website")
        .interact_text()?;

    Ok(description)
}

fn get_website_keywords() -> Result<String, std::io::Error> {
    let keywords: String = Input::new()
        .with_prompt("Enter keywords (comma-separated)")
        .interact_text()?;

    Ok(keywords)
}

fn get_website_pages() -> Result<String, std::io::Error> {
    let navbar_items: String = Input::new()
        .with_prompt("Enter navbar items (comma-separated)")
        .interact_text()?;

    Ok(navbar_items)
}

fn sanitise_string(page_name: &str) -> String {
    let sanitised_page_name = page_name
        .trim()
        .to_lowercase()
        .replace(" ", "_")
        .to_string();
    return sanitised_page_name;
}

fn generate_navbar_list(navbar_items: String) -> String {
    let navbar_items: Vec<&str> = navbar_items.split(',').collect();
    return navbar_items
        .iter()
        .map(|item| item.trim())
        .map(|item| {
            format!(
                "<li class=\"navbar__link\"><a href=\"/{}\">{}</a></li>",
                item, item
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
}

pub fn init_project() -> Result<(), std::io::Error> {
    let project_name: String = get_project_name()?;
    let project_dir = format!("./{}", sanitise_string(&project_name));
    if fs::metadata(&project_dir).is_ok() {
        println!("[WARNING] directory '{project_name}' already exists");
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Operation canceled.",
        ));
    }

    let static_directory = &format!("{project_name}/static");
    let templates_directory = &format!("{project_name}/templates");
    let pages_directory = &format!("{project_name}/pages");

    fs::create_dir_all(sanitise_string(&project_name))?;
    fs::create_dir_all(static_directory)?;
    fs::create_dir_all(templates_directory)?;
    fs::create_dir_all(pages_directory)?;

    let logo_path = format!("{}/logo.png", static_directory);
    fs::write(logo_path, LOGO)?;
    let styles_path = format!("{}/styles.css", static_directory);
    fs::write(styles_path, STYLES)?;

    let title: String = get_website_title()?;
    let author: String = get_website_author()?;
    let description: String = get_website_description()?;
    let keywords: String = get_website_keywords()?;
    let pages: String = get_website_pages()?;

    let home_page = format!("{}/index.md", pages_directory);
    let home_page_content = "# Add your home page content here";
    let mut home_page_file = File::create(home_page)?;
    home_page_file.write_all(home_page_content.as_bytes())?;

    // create the blank markdown pages
    for page in pages.split(',') {
        let page_directory = format!("{}/pages/{}", project_name, sanitise_string(page));
        fs::create_dir_all(page_directory.clone())?;
        let page_path = format!("{}/index.md", page_directory);
        let page_content = "# Add your content here";
        let mut page_file = File::create(page_path)?;
        page_file.write_all(page_content.as_bytes())?;
    }

    let navbar_list = generate_navbar_list(pages);
    let timestamp = get_timestamp();

    let html_content = format!(
        "<!DOCTYPE html>
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
            <link rel=\"stylesheet\" href=\"/static/styles.css?v={timestamp}\" />
            <link rel=\"shortcut icon\" type=\"image/x-icon\" href=\"/static/logo.png?v={timestamp}\" />
            <title>{title}</title>
            <meta name=\"description\" content=\"{description}\" />
            <meta name=\"author\" content=\"{author}\" />
            <meta name=\"keywords\" content=\"{keywords}\" />
            <!-- Add more fields here as needed -->
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
                                {navbar_list}
                            </ul>
                        </nav>
                    </header>
                    <main></main>
                    <footer class=\"footer\">
                        <p>&copy; 2023 {author}. All Rights Reserved.</p>
                    </footer>
                </div>
            </div>
        </body>
        </html>",
    );

    let html_path = Path::new(templates_directory).join("template.html");
    fs::write(&html_path, html_content)?;

    Ok(())
}
