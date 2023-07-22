use std::fs;
use std::path::Path;

pub fn init_project() -> Result<(), std::io::Error> {
    let src_directory = "src";
    let dist_directory = "dist";

    // Create the source directory
    fs::create_dir_all(src_directory)?;

    // Create the destination directory
    fs::create_dir_all(dist_directory)?;

    // Create a sample Markdown file (optional)
    let sample_md_content = "# Welcome to My SSG Project\n\nStart writing your content here!";
    let sample_md_path = Path::new(src_directory).join("index.md");
    fs::write(&sample_md_path, sample_md_content)?;

    Ok(())
}
