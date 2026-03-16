use std::process::Command;
use std::io;

pub enum CommandStatus {
    Success = 0,
    FatalError = 1,
}

pub fn run_git_status() -> io::Result<String> {
    let output = Command::new("git")
        .args(["status", "--short"])
        .output()?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Git command failed: {}", error),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn run_cs_fix(files: &[String]) -> io::Result<bool> {
    if files.is_empty() {
        return Ok(true);
    }

    for file in files {
        let args = build_cs_fix_args(file);

        let status = Command::new("docker")
            .args(&args)
            .status()?;

        if !status.success() {
            return Ok(false);
        }
        println!("---------------------END TO FILE: {}.-----------------------", file.to_string());
    }

    Ok(true)
}

pub fn run_composer_stan() -> io::Result<bool> {
    let container = "ninja_symfony";
    let args = vec![
        "exec".to_string(),
        container.to_string(),
        "composer".to_string(),
        "stan".to_string(),
    ];

    let status = Command::new("docker")
        .args(&args)
        .status()?;

    Ok(status.success())
}

pub fn run_test_command(command_str: &str) -> io::Result<bool> {
    let parts: Vec<&str> = command_str.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(false);
    }

    let mut cmd = Command::new(parts[0]);
    if parts.len() > 1 {
        cmd.args(&parts[1..]);
    }

    let status = cmd.status()?;

    Ok(status.success())
}

fn build_cs_fix_args(file: &str) -> Vec<String> {
    let container = "ninja_symfony";
    println!("Linting command for file: {}", file);
    vec![
        "exec".to_string(),
        container.to_string(),
        "composer".to_string(),
        "cs:fix".to_string(),
        file.to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_status_values() {
        assert_eq!(CommandStatus::Success as i32, 0);
        assert_eq!(CommandStatus::FatalError as i32, 1);
    }

    #[test]
    fn test_build_cs_fix_args() {
        let args = build_cs_fix_args("file1.php");
        assert_eq!(
            args,
            vec![
                "exec".to_string(),
                "ninja_symfony".to_string(),
                "composer".to_string(),
                "cs:fix".to_string(),
                "file1.php".to_string(),
            ]
        );
    }

    #[test]
    fn test_run_composer_stan_command_exists() {
        // This is a minimal test to ensure the logic exists
        // Actual execution would require docker
    }

    #[test]
    fn test_run_test_command_empty() {
        let result = run_test_command("");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}

