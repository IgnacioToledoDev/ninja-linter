use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use colored::Colorize;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub test_command: Option<String>,
}

const CONFIG_FILE: &str = ".ninja-linter.json";

impl Config {
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string(CONFIG_FILE) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
        Config::default()
    }

    pub fn save(&self) -> io::Result<()> {
        let content = serde_json::to_string_pretty(self).unwrap();
        fs::write(CONFIG_FILE, content)
    }

    pub fn get_or_set_test_command(&mut self) -> String {
        if let Some(ref cmd) = self.test_command {
            return cmd.clone();
        }

        println!("{}", "No test command configured.".yellow());
        print!("Please enter the test command (e.g., 'docker exec ninja_symfony bin/phpunit'): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let input = input.trim().to_string();

        self.test_command = Some(input.clone());
        if let Err(e) = self.save() {
            eprintln!("{}: {}", "Error saving config".red(), e);
        }

        input
    }
}
