use pulldown_cmark::{html, Parser};
use std::{
    fs::{self, DirEntry},
    path::Path,
};

fn convert_md_to_html(md_content: &str) -> String {
    let parser = Parser::new(md_content);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    return html_output;
}

fn read_all_files_and_dirs_recursive(
    dir: &Path,
    files: &mut Vec<DirEntry>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            read_all_files_and_dirs_recursive(&path, files)?;
        } else if path.is_file() {
            files.push(entry);
        }
    }

    Ok(())
}

pub fn build_project() -> Result<(), Box<dyn std::error::Error>> {
    let dist_path = format!("dist");
    fs::create_dir(&dist_path)?;

    let pages_dir = format!("pages");
    let mut pages = Vec::new();
    read_all_files_and_dirs_recursive(Path::new(&pages_dir), &mut pages)?;
    println!("pages: {:?}", pages);

    for page in pages {
        let template_path = format!("templates/template.html");
        let template_content = fs::read_to_string(&template_path)?;
        let page_path = page.path();
        let relative_path = page_path.strip_prefix(Path::new(&pages_dir))?;
        let relative_path_string = relative_path.to_string_lossy();

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
            let dist_file_path = format!("dist/{}", relative_path_string);
            println!("dist_file_path: {}", dist_file_path);
            fs::write(&dist_file_path, updated_template)?;
        }
    }

    Ok(())
}
