use crate::init;
use clap::{App, SubCommand};

pub fn parse_arguments() -> Result<(), String> {
    let matches = App::new("SSG - Static Site Generator")
        .version("1.0")
        .author("Radley E. Sidwell-Lewis")
        .about("A static site generator written in Rust")
        .subcommand(SubCommand::with_name("init").about("Initialise a new SSG project"))
        .get_matches();

    if let Some(("init", _)) = matches.subcommand() {
        if init::init_project().is_ok() {
            println!("SSG project initialised successfully.");
        } else {
            return Err("Failed to initialise SSG project.".to_string());
        }
    } else {
        // Handle other commands or show help message here
        println!("Use 'ssg init' to initialise a new SSG project.");
    }

    Ok(())
}
