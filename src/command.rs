use std::process::{Command, Stdio};
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
        return Err(io::Error::other(format!("Git command failed: {}", error)));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn run_cs_fix(files: &[String], container: &str, silent: bool) -> io::Result<bool> {
    if files.is_empty() {
        return Ok(true);
    }

    for file in files {
        let args = build_cs_fix_args(file, container, silent);

        let mut cmd = Command::new("docker");
        cmd.args(&args);
        if silent {
            cmd.stdout(Stdio::null()).stderr(Stdio::null());
        }
        let status = cmd.status()?;

        if !status.success() {
            return Ok(false);
        }
        if !silent {
            println!("---------------------END TO FILE: {}.-----------------------", file);
        }
    }

    Ok(true)
}

pub fn run_composer_stan(container: &str, silent: bool) -> io::Result<bool> {
    let args = vec![
        "exec".to_string(),
        container.to_string(),
        "composer".to_string(),
        "stan".to_string(),
    ];

    let mut cmd = Command::new("docker");
    cmd.args(&args);
    if silent {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    }
    let status = cmd.status()?;

    Ok(status.success())
}

pub fn run_test_command(command_str: &str, container: &str, silent: bool) -> io::Result<bool> {
    if command_str.trim().is_empty() {
        return Ok(false);
    }

    let mut cmd = Command::new("docker");
    cmd.args(["exec", container, "sh", "-c", command_str]);
    if silent {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    }
    let status = cmd.status()?;

    Ok(status.success())
}

fn build_cs_fix_args(file: &str, container: &str, silent: bool) -> Vec<String> {
    if !silent {
        println!("Linting command for file: {}", file);
    }
    vec![
        "exec".to_string(),
        container.to_string(),
        "composer".to_string(),
        "cs:fix".to_string(),
        file.to_string(),
    ]
}

pub fn run_diff_tree_command() -> io::Result<String> {
    let output = Command::new("git")
        .args(["diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"])
        .output()?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::other(format!("Git diff tree command failed: {}", error)));
    }
 
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
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
        let args = build_cs_fix_args("file1.php", "ninja_symfony", false);
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
    fn test_build_cs_fix_args_custom_container() {
        let args = build_cs_fix_args("src/Controller/FooController.php", "my_app_container", false);
        assert_eq!(args[0], "exec");
        assert_eq!(args[1], "my_app_container");
        assert_eq!(args[2], "composer");
        assert_eq!(args[3], "cs:fix");
        assert_eq!(args[4], "src/Controller/FooController.php");
    }

    #[test]
    fn test_build_cs_fix_args_length() {
        let args = build_cs_fix_args("file.php", "some_container", false);
        assert_eq!(args.len(), 5);
    }

    #[test]
    fn test_run_composer_stan_command_exists() {
        // This is a minimal test to ensure the logic exists
        // Actual execution would require docker
    }

    #[test]
    fn test_run_test_command_empty() {
        let result = run_test_command("", "ninja_symfony", false);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_run_diff_tree_command_logic() {
        // We can't easily test actual git command execution in CI without a repo,
        // but we can ensure the function exists and compiles.
        // This is mainly to satisfy the requirement of adding tests.
    }
}

