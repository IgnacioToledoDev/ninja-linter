use chrono::{DateTime, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub test_command: Option<String>,
    pub container_name: Option<String>,
    #[serde(default)]
    pub updated_check_at: String,
}

const CONFIG_DIR: &str = "ninja-linter";
const CONFIG_FILENAME: &str = "config.json";

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(CONFIG_DIR)
        .join(CONFIG_FILENAME)
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
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self).unwrap();
        fs::write(path, content)
    }

    pub fn get_or_set_test_command(&mut self) -> String {
        if let Some(ref cmd) = self.test_command {
            return cmd.clone();
        }

        println!("{}", "No test command configured.".yellow());
        print!("Please enter the test command (e.g., 'docker exec ninja_symfony bin/phpunit'): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
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
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim().to_string();

        self.container_name = Some(input.clone());
        if let Err(e) = self.save() {
            eprintln!("{}: {}", "Error saving config".red(), e);
        }

        input
    }

    pub fn set_updated_check_at(&mut self, uca: DateTime<Utc>) {
        self.updated_check_at = uca.to_rfc3339();

        if let Err(e) = self.save() {
            eprint!("{}: {}", "Error saving file config".red(), e);
        }
    }

    pub fn get_updated_check_at(&self) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(&self.updated_check_at)
            .unwrap()
            .with_timezone(&Utc)
    }

    pub fn has_updated_check_at(&self) -> bool {
        !self.updated_check_at.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_path_ends_with_expected_components() {
        let path = config_path();
        let mut components = path.components().rev();
        assert_eq!(components.next().unwrap().as_os_str(), CONFIG_FILENAME);
        assert_eq!(components.next().unwrap().as_os_str(), CONFIG_DIR);
    }

    #[test]
    fn test_config_path_is_absolute() {
        let path = config_path();
        assert!(path.is_absolute());
    }

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
            ..Config::default()
        };
        let name = config.get_or_set_container_name();
        assert_eq!(name, "my_container");
    }

    #[test]
    fn test_container_name_not_mutated_when_already_set() {
        let mut config = Config {
            container_name: Some("original_container".to_string()),
            test_command: None,
            ..Config::default()
        };
        config.get_or_set_container_name();
        assert_eq!(
            config.container_name,
            Some("original_container".to_string())
        );
    }

    #[test]
    fn test_config_serializes_container_name() {
        let config = Config {
            container_name: Some("ninja_symfony".to_string()),
            test_command: None,
            ..Config::default()
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
        assert_eq!(
            config.test_command,
            Some("docker exec ninja_symfony bin/phpunit".to_string())
        );
    }

    #[test]
    fn test_config_roundtrip_with_both_fields() {
        let original = Config {
            container_name: Some("my_app_container".to_string()),
            test_command: Some("docker exec my_app bin/phpunit".to_string()),
            ..Config::default()
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.container_name, original.container_name);
        assert_eq!(restored.test_command, original.test_command);
    }

    // --- updated_check_at ---

    #[test]
    fn test_config_default_updated_check_at_is_empty() {
        let config = Config::default();
        assert!(config.updated_check_at.is_empty());
    }

    #[test]
    fn test_has_updated_check_at_false_when_empty() {
        let config = Config::default();
        assert!(!config.has_updated_check_at());
    }

    #[test]
    fn test_has_updated_check_at_true_when_set() {
        let config = Config {
            updated_check_at: "2026-07-07T00:00:00+00:00".to_string(),
            ..Config::default()
        };
        assert!(config.has_updated_check_at());
    }

    #[test]
    fn test_set_updated_check_at_stores_rfc3339() {
        let mut config = Config::default();
        let dt: chrono::DateTime<Utc> = chrono::DateTime::from_timestamp(0, 0).unwrap();
        config.updated_check_at = dt.to_rfc3339();
        assert_eq!(config.updated_check_at, "1970-01-01T00:00:00+00:00");
    }

    #[test]
    fn test_get_updated_check_at_roundtrips() {
        let mut config = Config::default();
        let dt: chrono::DateTime<Utc> = chrono::DateTime::from_timestamp(1_000_000, 0).unwrap();
        config.updated_check_at = dt.to_rfc3339();
        let parsed = config.get_updated_check_at();
        assert_eq!(parsed, dt);
    }

    #[test]
    fn test_config_serializes_updated_check_at() {
        let config = Config {
            updated_check_at: "2026-07-07T00:00:00+00:00".to_string(),
            ..Config::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("updated_check_at"));
        assert!(json.contains("2026-07-07T00:00:00+00:00"));
    }

    #[test]
    fn test_config_deserializes_updated_check_at() {
        let json = r#"{"test_command":null,"container_name":null,"updated_check_at":"2026-07-07T00:00:00+00:00"}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.updated_check_at, "2026-07-07T00:00:00+00:00");
    }

    #[test]
    fn test_config_backwards_compat_missing_updated_check_at() {
        let json = r#"{"test_command":null,"container_name":"ninja_symfony"}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(config.updated_check_at.is_empty());
    }

    #[test]
    fn test_config_roundtrip_with_updated_check_at() {
        let original = Config {
            updated_check_at: "2026-07-07T12:30:00+00:00".to_string(),
            ..Config::default()
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.updated_check_at, original.updated_check_at);
    }
}
