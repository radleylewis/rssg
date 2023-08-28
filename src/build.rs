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

    for page in pages {
        let template_path = format!("templates/template.html");
        let template_content = fs::read_to_string(&template_path)?;

        let page_path = page.path();
        let filename_with_extension = page_path
            .file_name()
            .expect("File name not available")
            .to_string_lossy();

        let mut parts = filename_with_extension.splitn(2, '.');
        let filename = parts.next().unwrap_or("");
        let new_file_name = format!("{}.{}", filename, "html");
        let relative_path = page_path.strip_prefix(Path::new(&pages_dir))?;
        let relative_directory = relative_path.parent().unwrap();
        if let Some(extension) = page_path.extension() {
            let content = fs::read_to_string(&page_path)?;
            let html_content: String;
            if extension == "md" {
                html_content = convert_md_to_html(&content);
            } else {
                html_content = content;
            }
            let updated_template = template_content.replace("<main></main>", &html_content);
            let dist_path = format!("dist/{}", relative_directory.to_string_lossy());
            fs::create_dir_all(&dist_path)?;
            let dist_file_path = format!("{}/{}", dist_path, new_file_name);
            fs::write(&dist_file_path, updated_template)?;
        }
    }

    Ok(())
}
