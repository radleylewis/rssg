mod build;
mod cli;
mod frontmatter;
mod highlight;
mod images;
mod init;
mod render;
mod serve;
mod utils;

fn main() {
    if let Err(e) = cli::parse_arguments() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
