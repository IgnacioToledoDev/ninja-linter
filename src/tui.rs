//! Terminal User Interface powered by [ratatui].
//!
//! This module owns the full lifecycle of the alternate-screen dashboard shown
//! while `--parallel` tasks run.  The single public entry-point is
//! [`run_dashboard`]: it blocks until every task finishes (or the user presses
//! `q` / `Esc`) and then returns whether all tasks succeeded.
//!
//! ## Layout
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                   🥷 Ninja Linter                   │
//! │  ┌─ Tasks ──────────────────────────────────────┐   │
//! │  │  ✅  Tests                                   │   │
//! │  │  🔄  CS Fixer                                │   │
//! │  │  ⏳  PHPStan                                 │   │
//! │  └──────────────────────────────────────────────┘   │
//! │  ┌─ PHP Files ──────────────────────────────────┐   │
//! │  │  📄  src/Controller/HomeController.php        │   │
//! │  │  📄  src/Entity/User.php                     │   │
//! │  └──────────────────────────────────────────────┘   │
//! │   Press q or Esc to exit early                      │
//! └─────────────────────────────────────────────────────┘
//! ```

use std::io::{self, Stdout};
use std::sync::mpsc;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

use crate::parallel::{TaskState, TaskStatus, TaskUpdate};

// ─── Type alias ──────────────────────────────────────────────────────────────

type CrosstermTerminal = Terminal<CrosstermBackend<Stdout>>;

// ─── App state ───────────────────────────────────────────────────────────────

/// All mutable state the dashboard needs to render each frame.
struct App {
    tasks: Vec<TaskState>,
    php_files: Vec<String>,
}

impl App {
    fn new(tasks: Vec<TaskState>, php_files: Vec<String>) -> Self {
        Self { tasks, php_files }
    }

    /// Apply a single update message sent by a worker thread.
    fn apply(&mut self, update: TaskUpdate) {
        match update {
            TaskUpdate::Started(i) => self.tasks[i].status = TaskStatus::Running,
            TaskUpdate::Finished(i, true) => self.tasks[i].status = TaskStatus::Done,
            TaskUpdate::Finished(i, false) => self.tasks[i].status = TaskStatus::Failed,
        }
    }

    /// `true` when every task has reached a terminal state (`Done` or `Failed`).
    fn is_complete(&self) -> bool {
        self.tasks
            .iter()
            .all(|t| matches!(t.status, TaskStatus::Done | TaskStatus::Failed))
    }

    /// `true` if at least one task ended in `Failed`.
    fn has_failure(&self) -> bool {
        self.tasks.iter().any(|t| t.status == TaskStatus::Failed)
    }
}

// ─── Rendering ───────────────────────────────────────────────────────────────

/// Top-level draw function — called every tick.
fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let outer = Block::default()
        .title(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "🥷 Ninja Linter",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
        ]))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let task_height = (app.tasks.len() as u16).saturating_add(2); // list rows + 2 border lines

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(task_height),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(inner);

    render_tasks(frame, app, chunks[0]);
    render_files(frame, app, chunks[1]);
    render_hint(frame, chunks[2]);
}

fn render_tasks(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .tasks
        .iter()
        .map(|task| {
            let (icon, style) = task_style(&task.status);
            ListItem::new(Line::from(vec![
                Span::raw(format!("  {icon} ")),
                Span::styled(task.label.clone(), style),
            ]))
        })
        .collect();

    let block = Block::default()
        .title(" Tasks ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    frame.render_widget(List::new(items).block(block), area);
}

fn render_files(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .php_files
        .iter()
        .map(|f| {
            ListItem::new(Line::from(vec![
                Span::raw("  📄 "),
                Span::styled(f.as_str(), Style::default().fg(Color::White)),
            ]))
        })
        .collect();

    let block = Block::default()
        .title(" PHP Files ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    frame.render_widget(List::new(items).block(block), area);
}

fn render_hint(frame: &mut Frame, area: Rect) {
    let hint = Paragraph::new("  Press q or Esc to exit early")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hint, area);
}

/// Returns the (icon, [`Style`]) pair for a given [`TaskStatus`].
fn task_style(status: &TaskStatus) -> (&'static str, Style) {
    match status {
        TaskStatus::Pending => ("⏳", Style::default().fg(Color::DarkGray)),
        TaskStatus::Running => (
            "🔄",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        TaskStatus::Done => ("✅", Style::default().fg(Color::Green)),
        TaskStatus::Failed => (
            "❌",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ),
    }
}

// ─── Terminal lifecycle ───────────────────────────────────────────────────────

fn setup_terminal() -> io::Result<CrosstermTerminal> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn restore_terminal(terminal: &mut CrosstermTerminal) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

// ─── Event loop ──────────────────────────────────────────────────────────────

const TICK_RATE: Duration = Duration::from_millis(100);

/// Drive the ratatui event loop until all tasks are done or the user exits.
fn run_loop(
    terminal: &mut CrosstermTerminal,
    app: &mut App,
    rx: &mpsc::Receiver<TaskUpdate>,
) -> io::Result<bool> {
    loop {
        // Drain every pending update without blocking.
        while let Ok(update) = rx.try_recv() {
            app.apply(update);
        }

        terminal.draw(|f| draw(f, app))?;

        // All tasks have reached a terminal state — render a final frame,
        // pause briefly so the user can read the result, then exit.
        if app.is_complete() {
            terminal.draw(|f| draw(f, app))?;
            std::thread::sleep(Duration::from_millis(600));
            return Ok(!app.has_failure());
        }

        // Non-blocking keyboard check (reuses TICK_RATE as the poll timeout).
        if event::poll(TICK_RATE)?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
            && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
        {
            return Ok(!app.has_failure());
        }
    }
}

// ─── Public API ──────────────────────────────────────────────────────────────

/// Launch the live dashboard in the terminal's alternate screen.
///
/// Worker threads are expected to have been spawned **before** this call.
/// Their progress is communicated via `rx`.  This function blocks until every
/// task reaches `Done` or `Failed`, or until the user presses `q` / `Esc`.
///
/// The terminal is fully restored (raw mode disabled, alternate screen left)
/// before this function returns, even on error.
///
/// # Returns
///
/// `true` if every task succeeded, `false` if any task failed or the terminal
/// could not be initialised.
pub fn run_dashboard(
    tasks: Vec<TaskState>,
    php_files: Vec<String>,
    rx: mpsc::Receiver<TaskUpdate>,
) -> bool {
    let mut terminal = match setup_terminal() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to initialise terminal: {e}");
            return false;
        }
    };

    let mut app = App::new(tasks, php_files);
    let result = run_loop(&mut terminal, &mut app, &rx);

    if let Err(e) = restore_terminal(&mut terminal) {
        eprintln!("Failed to restore terminal: {e}");
    }

    result.unwrap_or(false)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parallel::{TaskState, TaskStatus, TaskUpdate};

    fn make_app(statuses: &[TaskStatus]) -> App {
        let tasks = statuses
            .iter()
            .map(|s| TaskState {
                label: "task".to_string(),
                status: s.clone(),
            })
            .collect();
        App::new(tasks, vec!["src/Foo.php".to_string()])
    }

    #[test]
    fn test_is_complete_all_done() {
        let app = make_app(&[TaskStatus::Done, TaskStatus::Done]);
        assert!(app.is_complete());
    }

    #[test]
    fn test_is_complete_with_failure() {
        let app = make_app(&[TaskStatus::Done, TaskStatus::Failed]);
        assert!(app.is_complete());
    }

    #[test]
    fn test_is_complete_still_running() {
        let app = make_app(&[TaskStatus::Done, TaskStatus::Running]);
        assert!(!app.is_complete());
    }

    #[test]
    fn test_is_complete_pending() {
        let app = make_app(&[TaskStatus::Pending]);
        assert!(!app.is_complete());
    }

    #[test]
    fn test_has_failure_true() {
        let app = make_app(&[TaskStatus::Done, TaskStatus::Failed]);
        assert!(app.has_failure());
    }

    #[test]
    fn test_has_failure_false() {
        let app = make_app(&[TaskStatus::Done, TaskStatus::Done]);
        assert!(!app.has_failure());
    }

    #[test]
    fn test_apply_started() {
        let mut app = make_app(&[TaskStatus::Pending]);
        app.apply(TaskUpdate::Started(0));
        assert!(matches!(app.tasks[0].status, TaskStatus::Running));
    }

    #[test]
    fn test_apply_finished_success() {
        let mut app = make_app(&[TaskStatus::Running]);
        app.apply(TaskUpdate::Finished(0, true));
        assert!(matches!(app.tasks[0].status, TaskStatus::Done));
    }

    #[test]
    fn test_apply_finished_failure() {
        let mut app = make_app(&[TaskStatus::Running]);
        app.apply(TaskUpdate::Finished(0, false));
        assert!(matches!(app.tasks[0].status, TaskStatus::Failed));
    }

    #[test]
    fn test_task_style_covers_all_variants() {
        let statuses = [
            TaskStatus::Pending,
            TaskStatus::Running,
            TaskStatus::Done,
            TaskStatus::Failed,
        ];
        for s in &statuses {
            let (icon, _style) = task_style(s);
            assert!(!icon.is_empty());
        }
    }
}
