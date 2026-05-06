use crate::build;
use crate::init;
use crate::serve;
use clap::{Arg, Command};

pub fn parse_arguments() -> Result<(), String> {
    let matches = Command::new("rssg")
        .version("1.0")
        .author("Radley E. Sidwell-Lewis")
        .about("A static site generator written in Rust")
        .subcommand(Command::new("init").about("Initialise a new project"))
        .subcommand(Command::new("build").about("Build the project"))
        .subcommand(Command::new("new").about("Create a new page in the current directory"))
        .subcommand(
            Command::new("serve")
                .about("Serve the built site locally")
                .arg(
                    Arg::new("port")
                        .short('p')
                        .long("port")
                        .default_value("8080")
                        .help("Port to listen on"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("init", _)) => {
            if init::init_project().is_ok() {
                println!("Project initialised successfully.");
            } else {
                return Err("Failed to initialise project.".to_string());
            }
        }
        Some(("build", _)) => {
            if build::build_project().is_ok() {
                println!("Project built successfully.");
            } else {
                return Err("Failed to build project.".to_string());
            }
        }
        Some(("new", _)) => {
            if init::new_page().is_ok() {
                println!("Page created successfully.");
            } else {
                return Err("Failed to create page.".to_string());
            }
        }
        Some(("serve", sub)) => {
            let port: u16 = sub
                .get_one::<String>("port")
                .map(|s| s.as_str())
                .unwrap_or("8080")
                .parse()
                .map_err(|_| "Invalid port number".to_string())?;
            serve::serve(port).map_err(|e| e.to_string())?;
        }
        _ => {
            println!("Use 'rssg init' to initialise a new project.");
        }
    }

    Ok(())
}
