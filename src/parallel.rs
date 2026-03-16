use std::io::{stdout, IsTerminal, Write};
use std::sync::mpsc;
use std::thread;
use crossterm::{cursor, execute};
use colored::Colorize;

use crate::command::{run_cs_fix, run_composer_stan, run_test_command};

#[derive(Clone, PartialEq)]
enum TaskStatus {
    Pending,
    Running,
    Done,
    Failed,
}

struct TaskState {
    label: String,
    status: TaskStatus,
}

enum TaskUpdate {
    Started(usize),
    Finished(usize, bool),
}

fn render_task_list(tasks: &[TaskState], first_render: bool) {
    let is_tty = stdout().is_terminal();
    let active_count = tasks.len();

    if !first_render && is_tty && active_count > 0 {
        execute!(stdout(), cursor::MoveUp(active_count as u16)).ok();
    }

    for task in tasks {
        let line = match task.status {
            TaskStatus::Pending => format!("⏳ {}", task.label),
            TaskStatus::Running => format!("🔄 {} ...", task.label.yellow()),
            TaskStatus::Done => format!("✅ {}", task.label.green()),
            TaskStatus::Failed => format!("❌ {}", task.label.red()),
        };
        println!("{}", line);
    }
    stdout().flush().ok();
}

pub fn run_parallel(
    php_files: Vec<String>,
    container: String,
    test_command: Option<String>,
    run_stan: bool,
) -> bool {
    let mut tasks = vec![
        TaskState {
            label: "Tests".to_string(),
            status: if test_command.is_none() { TaskStatus::Done } else { TaskStatus::Pending },
        },
        TaskState {
            label: "CS Fixer".to_string(),
            status: TaskStatus::Pending,
        },
        TaskState {
            label: "PHPStan".to_string(),
            status: if !run_stan { TaskStatus::Done } else { TaskStatus::Pending },
        },
    ];

    render_task_list(&tasks, true);

    let (tx, rx) = mpsc::channel::<TaskUpdate>();

    if let Some(cmd) = test_command {
        let tx0 = tx.clone();
        thread::spawn(move || {
            tx0.send(TaskUpdate::Started(0)).ok();
            let result = run_test_command(&cmd).unwrap_or(false);
            tx0.send(TaskUpdate::Finished(0, result)).ok();
        });
    }

    {
        let tx1 = tx.clone();
        let files = php_files.clone();
        let cont = container.clone();
        thread::spawn(move || {
            tx1.send(TaskUpdate::Started(1)).ok();
            let result = run_cs_fix(&files, &cont, true).unwrap_or(false);
            tx1.send(TaskUpdate::Finished(1, result)).ok();
        });
    }

    if run_stan {
        let tx2 = tx.clone();
        let cont = container.clone();
        thread::spawn(move || {
            tx2.send(TaskUpdate::Started(2)).ok();
            let result = run_composer_stan(&cont).unwrap_or(false);
            tx2.send(TaskUpdate::Finished(2, result)).ok();
        });
    }

    drop(tx);

    for update in rx {
        match update {
            TaskUpdate::Started(i) => tasks[i].status = TaskStatus::Running,
            TaskUpdate::Finished(i, true) => tasks[i].status = TaskStatus::Done,
            TaskUpdate::Finished(i, false) => tasks[i].status = TaskStatus::Failed,
        }
        render_task_list(&tasks, false);
    }

    tasks.iter().all(|t| t.status != TaskStatus::Failed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_does_not_panic() {
        let tasks = vec![
            TaskState { label: "Pending task".to_string(), status: TaskStatus::Pending },
            TaskState { label: "Running task".to_string(), status: TaskStatus::Running },
            TaskState { label: "Done task".to_string(), status: TaskStatus::Done },
            TaskState { label: "Failed task".to_string(), status: TaskStatus::Failed },
        ];
        render_task_list(&tasks, true);
        render_task_list(&tasks, false);
    }

    #[test]
    fn test_task_status_all_done_returns_true() {
        let tasks = vec![
            TaskState { label: "Tests".to_string(), status: TaskStatus::Done },
            TaskState { label: "CS Fixer".to_string(), status: TaskStatus::Done },
            TaskState { label: "PHPStan".to_string(), status: TaskStatus::Done },
        ];
        let result = tasks.iter().all(|t| t.status != TaskStatus::Failed);
        assert!(result);
    }

    #[test]
    fn test_task_status_one_failed_returns_false() {
        let tasks = vec![
            TaskState { label: "Tests".to_string(), status: TaskStatus::Done },
            TaskState { label: "CS Fixer".to_string(), status: TaskStatus::Failed },
            TaskState { label: "PHPStan".to_string(), status: TaskStatus::Done },
        ];
        let result = tasks.iter().all(|t| t.status != TaskStatus::Failed);
        assert!(!result);
    }

    #[test]
    fn test_run_parallel_nothing_to_run() {
        let result = run_parallel(vec![], "ninja_symfony".to_string(), None, false);
        assert!(result);
    }
}
