mod command;
mod file;

use crate::command::{run_cs_fix, CommandStatus};
use crate::file::get_modified_files;
use clap::Parser;
use shadow_rs::shadow;
use std::process;
use colored::Colorize;
use log::{error};

shadow!(build);

#[derive(Parser, Debug)]
#[command(
    author = "@IgnacioToledoDev",
    version = build::PKG_VERSION,
    about = "Lints PHP files with cs-fixer in a docker container",
    long_about = None
)]
struct Args {
    // EMPTY
}

fn main() {
    Args::parse();

    if build::BRANCH.is_empty() {
        eprintln!("{}", "Error: No branch found!".red());
        process::exit(CommandStatus::FatalError as i32);
    }

    let php_files = match get_modified_files() {
        Ok(files) => files,
        Err(e) => {
            println!("{}", "Error getting modified files".red());
            error!("Error: in git status when run command {e}");
            process::exit(CommandStatus::FatalError as i32);
        }
    };

    if php_files.is_empty() {
        println!("\n {}", "No PHP files modified".green());
        process::exit(CommandStatus::Success as i32);
    }

    match run_cs_fix(&php_files) {
        Ok(true) => finish_process(),
        Ok(false) => {
            eprintln!("Error: cs-fixer failed to clean some files");
            process::exit(CommandStatus::FatalError as i32);
        }
        Err(e) => {
            println!("{}", "Error running cs-fixer".red());
            error!("Error: in cs:fix when run command {e}");
            process::exit(CommandStatus::FatalError as i32);
        }
    }
}

fn finish_process() {
    println!("{}", "✅ All PHP files are clean".underline().bold());
    process::exit(CommandStatus::Success as i32);
}
