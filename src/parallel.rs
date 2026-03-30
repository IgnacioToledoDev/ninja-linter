//! Parallel execution of linting tasks.
//!
//! Spawns up to three worker threads (tests, CS Fixer, PHPStan) concurrently
//! and delegates live status rendering to [`crate::tui`].

use std::sync::mpsc;
use std::thread;

use crate::command::{run_cs_fix, run_composer_stan, run_test_command};
use crate::tui;

// ─── Task types (public so `tui` can reference them) ─────────────────────────

#[derive(Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Done,
    Failed,
}

pub struct TaskState {
    pub label: String,
    pub status: TaskStatus,
}

pub enum TaskUpdate {
    Started(usize),
    Finished(usize, bool),
}

// ─── Public entry-point ───────────────────────────────────────────────────────

/// Run CS Fixer, optional tests, and optional PHPStan in parallel threads,
/// displaying a live ratatui dashboard while they execute.
///
/// # Returns
///
/// `true` if every enabled task succeeded, `false` otherwise.
pub fn run_parallel(
    php_files: Vec<String>,
    container: String,
    test_command: Option<String>,
    run_stan: bool,
) -> bool {
    let tasks = build_initial_tasks(&test_command, run_stan);
    let (tx, rx) = mpsc::channel::<TaskUpdate>();

    spawn_workers(&php_files, &container, test_command, run_stan, tx);

    tui::run_dashboard(tasks, php_files, rx)
}

// ─── Private helpers ──────────────────────────────────────────────────────────

fn build_initial_tasks(test_command: &Option<String>, run_stan: bool) -> Vec<TaskState> {
    vec![
        TaskState {
            label: "Tests".to_string(),
            status: if test_command.is_none() {
                TaskStatus::Done
            } else {
                TaskStatus::Pending
            },
        },
        TaskState {
            label: "CS Fixer".to_string(),
            status: TaskStatus::Pending,
        },
        TaskState {
            label: "PHPStan".to_string(),
            status: if !run_stan {
                TaskStatus::Done
            } else {
                TaskStatus::Pending
            },
        },
    ]
}

fn spawn_workers(
    php_files: &[String],
    container: &str,
    test_command: Option<String>,
    run_stan: bool,
    tx: mpsc::Sender<TaskUpdate>,
) {
    if let Some(cmd) = test_command {
        let tx0 = tx.clone();
        let cont = container.to_string();
        thread::spawn(move || {
            tx0.send(TaskUpdate::Started(0)).ok();
            let result = run_test_command(&cmd, &cont).unwrap_or(false);
            tx0.send(TaskUpdate::Finished(0, result)).ok();
        });
    }

    {
        let tx1 = tx.clone();
        let files = php_files.to_vec();
        let cont = container.to_string();
        thread::spawn(move || {
            tx1.send(TaskUpdate::Started(1)).ok();
            let ok = run_cs_fix(&files, &cont, true).unwrap_or(false);
            tx1.send(TaskUpdate::Finished(1, ok)).ok();
        });
    }

    if run_stan {
        let tx2 = tx.clone();
        let cont = container.to_string();
        thread::spawn(move || {
            tx2.send(TaskUpdate::Started(2)).ok();
            let ok = run_composer_stan(&cont).unwrap_or(false);
            tx2.send(TaskUpdate::Finished(2, ok)).ok();
        });
    }

    // Drop the original sender so the channel closes when all workers finish.
    drop(tx);
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_build_initial_tasks_all_disabled() {
        let tasks = build_initial_tasks(&None, false);
        assert_eq!(tasks.len(), 3);
        assert!(matches!(tasks[0].status, TaskStatus::Done)); // tests skipped
        assert!(matches!(tasks[1].status, TaskStatus::Pending)); // cs-fixer always pending
        assert!(matches!(tasks[2].status, TaskStatus::Done)); // stan skipped
    }

    #[test]
    fn test_build_initial_tasks_all_enabled() {
        let cmd = Some("bin/phpunit".to_string());
        let tasks = build_initial_tasks(&cmd, true);
        assert!(matches!(tasks[0].status, TaskStatus::Pending));
        assert!(matches!(tasks[1].status, TaskStatus::Pending));
        assert!(matches!(tasks[2].status, TaskStatus::Pending));
    }
}
