mod command;
mod file;
mod config;

use crate::command::{run_cs_fix, run_composer_stan, run_test_command, CommandStatus};
use crate::file::get_modified_files;
use crate::config::Config;
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
    #[arg(long, help = "Run composer stan after cs-fixer")]
    stan: bool,

    #[arg(short, long, help = "Run project tests before cs-fixer")]
    test: bool,
}

fn main() {
    let args = Args::parse();

    if build::BRANCH.is_empty() {
        eprintln!("{}", "Error: No branch found!".red());
        process::exit(CommandStatus::FatalError as i32);
    }

    let mut config = Config::load();
    let container = config.get_or_set_container_name();

    if args.test {
        run_tests(&mut config);
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

    match run_cs_fix(&php_files, &container) {
        Ok(true) => {
            if args.stan {
                run_stan(&container);
            }
            finish_process()
        },
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

fn run_stan(container: &str) {
    println!("{}", "Running composer stan...".yellow());
    match run_composer_stan(container) {
        Ok(true) => println!("{}", "✅ PHPStan passed".green()),
        Ok(false) => {
            eprintln!("{}", "❌ PHPStan failed".red());
            process::exit(CommandStatus::FatalError as i32);
        }
        Err(e) => {
            println!("{}", "Error running composer stan".red());
            error!("Error: in composer stan when run command {e}");
            process::exit(CommandStatus::FatalError as i32);
        }
    }
}

fn run_tests(config: &mut Config) {
    let command = config.get_or_set_test_command();

    println!("{}", format!("Running tests: {}...", command).yellow());
    match run_test_command(&command) {
        Ok(true) => println!("{}", "✅ Tests passed".green()),
        Ok(false) => {
            eprintln!("{}", "❌ Tests failed".red());
            process::exit(CommandStatus::FatalError as i32);
        }
        Err(e) => {
            println!("{}", "Error running tests".red());
            error!("Error: in tests when run command {e}");
            process::exit(CommandStatus::FatalError as i32);
        }
    }
}

fn finish_process() {
    println!("{}", "✅ All PHP files are clean".underline().bold());
    process::exit(CommandStatus::Success as i32);
}
