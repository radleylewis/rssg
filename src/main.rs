use std::fs;
use std::io;
use std::path::Path;

fn main() {
    let src_directory = "./files";
    let dist_directory = "./dist";

    match copy_files(src_directory, dist_directory) {
        Ok(_) => println!("Files copied successfully."),
        Err(err) => eprintln!("Error: {}", err),
    }
}

fn copy_files(src_directory: &str, dist_directory: &str) -> io::Result<()> {
    let src_path = Path::new(src_directory);
    let dist_path = Path::new(dist_directory);

    if !src_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Source directory does not exist.",
        ));
    }

    if !dist_path.exists() {
        fs::create_dir(dist_path)?;
    }

    for entry in fs::read_dir(src_path)? {
        let entry = entry?;
        let src_file_path = entry.path();

        let src_metadata = fs::metadata(&src_file_path)?;
        let dst_file_path = dist_path.join(src_file_path.file_name().unwrap());

        if src_metadata.is_file() {
            fs::copy(&src_file_path, &dst_file_path)?;
        }
    }

    Ok(())
}
