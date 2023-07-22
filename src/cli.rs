use crate::ssg;
use clap::{App, SubCommand};

pub fn parse_arguments() -> Result<(), String> {
    let matches = App::new("SSG - Static Site Generator")
        .version("1.0")
        .author("Your Name")
        .about("A static site generator written in Rust")
        .subcommand(SubCommand::with_name("init").about("Initialize a new SSG project"))
        .get_matches();

    if let Some(("init", _)) = matches.subcommand() {
        if ssg::init_project().is_ok() {
            println!("SSG project initialized successfully.");
        } else {
            return Err("Failed to initialize SSG project.".to_string());
        }
    } else {
        // Handle other commands or show help message here
        println!("Use 'ssg init' to initialize a new SSG project.");
    }

    Ok(())
}
