use pulldown_cmark::{html, Parser};
use std::fs;

fn convert_md_to_html(md_content: &str) -> String {
    let parser = Parser::new(md_content);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    return html_output;
}

pub fn build_project() -> Result<(), Box<dyn std::error::Error>> {
    let dist_path = format!("dist");
    fs::create_dir(&dist_path)?;

    let pages_dir = format!("pages");
    let page_pages = fs::read_dir(&pages_dir)?;

    for page in page_pages {
        let template_path = format!("templates/template.html");
        let template_content = fs::read_to_string(&template_path)?;
        if let Ok(page) = page {
            let page_path = page.path();
            let page_name = page_path.file_name().unwrap().to_string_lossy();

            if let Some(extension) = page_path.extension() {
                let extension = extension.to_string_lossy();

                let content = fs::read_to_string(&page_path)?;
                let html_content: String;
                if extension == "md" {
                    html_content = convert_md_to_html(&content);
                } else {
                    html_content = content;
                }

                let updated_template = template_content.replace("<main></main>", &html_content);
                let dist_file_path = format!("{}/{}", &dist_path, page_name);
                fs::write(&dist_file_path, updated_template)?;
            }
        }
    }

    Ok(())
}
