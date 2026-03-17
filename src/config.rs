use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use colored::Colorize;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub test_command: Option<String>,
    pub container_name: Option<String>,
}

const CONFIG_FILE: &str = ".ninja-linter.json";

fn config_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join(CONFIG_FILE)))
        .unwrap_or_else(|| PathBuf::from(CONFIG_FILE))
}

impl Config {
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string(config_path())
            && let Ok(config) = serde_json::from_str(&content)
        {
            return config;
        }
        Config::default()
    }

    pub fn save(&self) -> io::Result<()> {
        let content = serde_json::to_string_pretty(self).unwrap();
        fs::write(config_path(), content)
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

    pub fn get_or_set_container_name(&mut self) -> String {
        if let Some(ref name) = self.container_name {
            return name.clone();
        }

        println!("{}", "No Docker container configured.".yellow());
        print!("Please enter the Docker container name (e.g., 'ninja_symfony'): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let input = input.trim().to_string();

        self.container_name = Some(input.clone());
        if let Err(e) = self.save() {
            eprintln!("{}: {}", "Error saving config".red(), e);
        }

        input
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default_has_no_container_name() {
        let config = Config::default();
        assert!(config.container_name.is_none());
    }

    #[test]
    fn test_get_or_set_container_name_returns_existing() {
        let mut config = Config {
            container_name: Some("my_container".to_string()),
            test_command: None,
        };
        let name = config.get_or_set_container_name();
        assert_eq!(name, "my_container");
    }

    #[test]
    fn test_container_name_not_mutated_when_already_set() {
        let mut config = Config {
            container_name: Some("original_container".to_string()),
            test_command: None,
        };
        config.get_or_set_container_name();
        assert_eq!(config.container_name, Some("original_container".to_string()));
    }

    #[test]
    fn test_config_serializes_container_name() {
        let config = Config {
            container_name: Some("ninja_symfony".to_string()),
            test_command: None,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("ninja_symfony"));
        assert!(json.contains("container_name"));
    }

    #[test]
    fn test_config_deserializes_container_name_from_json() {
        let json = r#"{"test_command": null, "container_name": "ninja_symfony"}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.container_name, Some("ninja_symfony".to_string()));
    }

    #[test]
    fn test_config_backwards_compat_without_container_name() {
        // Old configs that don't have container_name should still parse fine
        let json = r#"{"test_command": "docker exec ninja_symfony bin/phpunit"}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(config.container_name.is_none());
        assert_eq!(config.test_command, Some("docker exec ninja_symfony bin/phpunit".to_string()));
    }

    #[test]
    fn test_config_roundtrip_with_both_fields() {
        let original = Config {
            container_name: Some("my_app_container".to_string()),
            test_command: Some("docker exec my_app bin/phpunit".to_string()),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.container_name, original.container_name);
        assert_eq!(restored.test_command, original.test_command);
    }
}
