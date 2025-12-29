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

    let args = build_cs_fix_args(files);

    let status = Command::new("docker")
        .args(&args)
        .status()?;

    Ok(status.success())
}

fn build_cs_fix_args(files: &[String]) -> Vec<String> {
    let container = "ninja_symfony";
    let mut args = vec![
        "exec".to_string(),
        container.to_string(),
        "composer".to_string(),
        "cs:fix".to_string(),
    ];
    for file in files {
        args.push(file.clone());
    }
    args
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
        let files = vec!["file1.php".to_string(), "file2.php".to_string()];
        let args = build_cs_fix_args(&files);
        assert_eq!(
            args,
            vec![
                "exec".to_string(),
                "ninja_symfony".to_string(),
                "composer".to_string(),
                "cs:fix".to_string(),
                "file1.php".to_string(),
                "file2.php".to_string(),
            ]
        );
    }

    #[test]
    fn test_build_cs_fix_args_empty() {
        let files: Vec<String> = Vec::new();
        let args = build_cs_fix_args(&files);
        assert_eq!(
            args,
            vec![
                "exec".to_string(),
                "ninja_symfony".to_string(),
                "composer".to_string(),
                "cs:fix".to_string(),
            ]
        );
    }
}

