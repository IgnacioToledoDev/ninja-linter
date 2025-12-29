mod command;
mod file;

use crate::command::{run_cs_fix, CommandStatus};
use crate::file::get_modified_files;
use clap::Parser;
use shadow_rs::shadow;
use std::process;

shadow!(build);

#[derive(Parser, Debug)]
#[command(
    author = "@IgnacioToledoDev",
    version = build::CLAP_LONG_VERSION,
    about = "Lints PHP files with cs-fixer in a docker container",
    long_about = None
)]
struct Args {
    // EMPTY
}

fn main() {
    Args::parse();

    if build::BRANCH.is_empty() {
        eprintln!("Error: No branch found!");
        process::exit(CommandStatus::FatalError as i32);
    }

    let php_files = match get_modified_files() {
        Ok(files) => files,
        Err(e) => {
            eprintln!("Error getting modified files: {}", e);
            process::exit(CommandStatus::FatalError as i32);
        }
    };

    if php_files.is_empty() {
        println!("No PHP files modified");
        process::exit(CommandStatus::Success as i32);
    }

    match run_cs_fix(&php_files) {
        Ok(true) => finish_process(),
        Ok(false) => {
            eprintln!("Error: cs-fixer failed to clean some files");
            process::exit(CommandStatus::FatalError as i32);
        }
        Err(e) => {
            eprintln!("Error running cs-fixer: {}", e);
            process::exit(CommandStatus::FatalError as i32);
        }
    }
}

fn finish_process() {
    println!("✅ All PHP files are clean");
    process::exit(CommandStatus::Success as i32);
}
