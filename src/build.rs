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

fn copy_directory(src: &Path, dest: &Path) -> Result<(), std::io::Error> {
    if !src.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Source is not a directory",
        ));
    }

    if !dest.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Destination is not a directory",
        ));
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap();
        let dest_path = dest.join(file_name);
        if path.is_dir() {
            fs::create_dir_all(&dest_path)?;
            copy_directory(&path, &dest_path)?;
        } else if path.is_file() {
            fs::copy(&path, &dest_path)?;
        }
    }

    Ok(())
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

fn add_active_id_to_navbar(html: &str, page_name: &str) -> String {
    let search_string = format!(
        "<a href=\"/{}\">{}</a>",
        page_name.to_lowercase(),
        page_name
    );
    let id_string = "id=\"active\"";
    if html.find(&search_string).is_some() {
        let modified_html = html.replace(
            &search_string,
            &format!(
                "<a href=\"{}\" {}>{}</a>",
                page_name.to_lowercase(),
                id_string,
                page_name
            ),
        );
        return modified_html;
    }
    return html.to_string();
}

pub fn build_project() -> Result<(), Box<dyn std::error::Error>> {
    let dist_path = format!("dist");
    if fs::metadata(dist_path.clone()).is_ok() {
        fs::remove_dir_all(dist_path.clone())?;
    }
    fs::create_dir_all(&dist_path)?;
    fs::create_dir_all(format!("{}/static", dist_path))?;

    let static_src = Path::new("./static");
    let static_dist = Path::new("./dist/static");
    copy_directory(static_src, static_dist)?;

    let pages_dir = format!("pages");
    let mut pages = Vec::new();
    read_all_files_and_dirs_recursive(Path::new(&pages_dir), &mut pages)?;

    for page in pages {
        let template_path = format!("templates/template.html");
        let og_template_content = fs::read_to_string(&template_path)?;

        let page_path = page.path();
        let parent = page_path.parent().unwrap().to_string_lossy();
        let current_page = parent.split("/").last().unwrap();

        let template_content = add_active_id_to_navbar(&og_template_content, current_page);

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
            let file_contents = fs::read_to_string(&page_path)?;
            let content: String;
            if extension == "md" {
                content = convert_md_to_html(&file_contents);
            } else {
                content = file_contents;
            }
            let updated_main = format!("<main><div class=\"content\">{}</div></main>", &content);
            let updated_template = template_content.replace("<main></main>", &updated_main);
            let dist_path = format!("dist/{}", relative_directory.to_string_lossy());
            fs::create_dir_all(&dist_path)?;
            let dist_file_path = format!("{}/{}", dist_path, new_file_name);
            fs::write(&dist_file_path, updated_template)?;
        }
    }

    Ok(())
}
