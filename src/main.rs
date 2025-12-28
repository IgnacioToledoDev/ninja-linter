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

// TODO: pending better handler of error
fn main() {
    Args::parse();

    if build::BRANCH.is_empty() {
        println!("No branch founded!");
        process::exit(CommandStatus::Failure as i32);
    }

    let php_files = get_modified_files(); // Get all modified files in the project
    if php_files.is_empty() {
        println!("No PHP files modified");
        process::exit(CommandStatus::Success as i32);
    }

    for file in php_files {
        if !run_cs_fix(&file) {
            eprintln!("Error running cs-fixer for file {}", file);
            process::exit(CommandStatus::FatalError as i32); // TODO: check if this is correct
        }
    }

    finish_process()
}

fn finish_process() {
    println!("✅ All PHP files are clean");
    process::exit(CommandStatus::Success as i32);
}
