use std::process::Command;

pub enum CommandStatus {
    Success = 0,
    FatalError = 1,
}

pub fn run_git_status() -> String {
    let (shell, flag) = get_shell();

    let output = Command::new(shell)
        .arg(flag)
        .arg("git status --short | cut -c4-")
        .output()
        .expect("Failed to execute command");

    String::from_utf8_lossy(&output.stdout).into_owned()
}

// TODO: Change this by is a bad practice to CLI
fn get_shell() -> (&'static str, &'static str) {
    let (shell, flag) = if cfg!(windows) {
        ("powershell", "-Command")
    } else {
        ("bash", "-c")
    };
    (shell, flag)
}

pub fn run_cs_fix(file: &str) -> bool {
    let container = "ninja_symfony";
    let status = Command::new("docker")
        .args(["exec", container, "composer", "cs:fix", file])
        .status()
        .expect("Failed to execute command");

    status.success()
}
