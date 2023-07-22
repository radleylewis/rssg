use dialoguer::Input;
use std::fs;
use std::path::Path;

pub fn init_project() -> Result<(), std::io::Error> {
    let src_directory = "site";
    let assets_directory = "site/assets";

    fs::create_dir_all(src_directory)?;
    fs::create_dir_all(assets_directory)?;

    let title: String = Input::new()
        .with_prompt("Enter the title of your new website")
        .interact_text()?;

    let description: String = Input::new()
        .with_prompt("Enter the description")
        .interact_text()?;

    let author: String = Input::new()
        .with_prompt("Enter the author")
        .interact_text()?;

    let keywords: String = Input::new()
        .with_prompt("Enter keywords (comma-separated)")
        .interact_text()?;

    let header_content = format!(
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
                <title>{}</title>
                <meta name=\"description\" content=\"{}\" />
                <meta name=\"author\" content=\"{}\" />
                <meta name=\"keywords\" content=\"{}\" />
                <!-- Add more fields here as needed -->
            </head>
        </html>",
        title, description, author, keywords
    );

    let header_path = Path::new(src_directory).join("header.html");
    fs::write(&header_path, header_content)?;

    let styles_path = Path::new(src_directory).join("assets/styles.css");
    fs::write(&styles_path, "")?;

    Ok(())
}
