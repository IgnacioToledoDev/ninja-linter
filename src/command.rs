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
    }

    Ok(true)
}

fn build_cs_fix_args(file: &str) -> Vec<String> {
    let container = "ninja_symfony";
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
}

