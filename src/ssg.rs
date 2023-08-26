use dialoguer::Input;
use std::fs;
use std::path::Path;

fn get_project_name() -> Result<String, std::io::Error> {
    let default_project_name: String = "my-project".to_string();
    let title: String = Input::new()
        .with_prompt("What is the name of your new project?")
        .default(default_project_name)
        .interact_text()?;

    Ok(title)
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

fn get_website_navbar_items() -> Result<String, std::io::Error> {
    let navbar_items: String = Input::new()
        .with_prompt("Enter navbar items (comma-separated)")
        .interact_text()?;

    Ok(navbar_items)
}

fn generate_navbar_list(navbar_items: String) -> String {
    let navbar_items: Vec<&str> = navbar_items.split(',').collect();
    return navbar_items
        .iter()
        .map(|item| item.trim())
        .map(|item| format!("<li><a href='/{}'>{}</a></li>", item, item))
        .collect::<Vec<_>>()
        .join("\n");
}

pub fn init_project() -> Result<(), std::io::Error> {
    let project_name: String = get_project_name()?;
    let assets_directory = &format!("{project_name}/assets");
    let templates_directory = &format!("{project_name}/templates");

    fs::create_dir_all(project_name)?;
    fs::create_dir_all(assets_directory)?;
    fs::create_dir_all(templates_directory)?;

    let title: String = get_website_title()?;
    let author: String = get_website_author()?;
    let description: String = get_website_description()?;
    let keywords: String = get_website_keywords()?;
    let navbar_items: String = get_website_navbar_items()?;

    let navbar_list = generate_navbar_list(navbar_items);

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
            <link rel=\"stylesheet\" href=\"./assets/styles.css\" />
            <link rel=\"shortcut icon\" type=\"image/x-icon\" href=\"./assets/favicon.png\" />
            <title>{title}</title>
            <meta name=\"description\" content=\"{description}\" />
            <meta name=\"author\" content=\"{author}\" />
            <meta name=\"keywords\" content=\"{keywords}\" />
            <!-- Add more fields here as needed -->
        </head>
        <body>
            <header>
                <nav>
                    <ul>
                        {navbar_list}
                    </ul>
                </nav>
            </header>
            <main></main>
            <footer>
                <p>&copy; {author}. </p>
            </footer>
        </body>
        </html>",
    );

    let html_path = Path::new(templates_directory).join("template.html");
    fs::write(&html_path, html_content)?;

    let styles_path = Path::new(assets_directory).join("styles.css");
    fs::write(&styles_path, "")?;

    Ok(())
}
