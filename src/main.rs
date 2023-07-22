mod cli;
mod ssg;

fn main() {
    if let Err(e) = cli::parse_arguments() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
