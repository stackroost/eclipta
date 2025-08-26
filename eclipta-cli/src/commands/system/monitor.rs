use chrono::{DateTime, Utc};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, time::Duration, path::PathBuf};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table},
    Terminal,
};

use crate::utils::paths::default_state_path;
use crate::utils::state::load_state;

struct AttachmentRow {
    name: String,
    kind: String,
    hook: String,
    pid: String,
    status: String,
    pinned: String,
    created: String,
}

fn proc_alive(pid: u32) -> bool {
    let p = PathBuf::from(format!("/proc/{}", pid));
    p.exists()
}

pub async fn handle_monitor() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        let st = load_state(&default_state_path());
        let rows_data: Vec<AttachmentRow> = st
            .attachments
            .iter()
            .map(|r| {
                let hook = match (&r.trace_category, &r.trace_name) {
                    (Some(c), Some(n)) => format!("{}:{}", c, n),
                    _ => r.kind.clone(),
                };
                let pinned = r
                    .pinned_prog
                    .as_ref()
                    .map(|p| if p.exists() { "yes" } else { "missing" })
                    .unwrap_or("no");
                let status = if proc_alive(r.pid) { "online" } else { "offline" };
                let created = DateTime::from_timestamp(r.created_at, 0)
                    .unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
                    .with_timezone(&Utc)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string();
                AttachmentRow {
                    name: r.name.clone(),
                    kind: r.kind.clone(),
                    hook,
                    pid: r.pid.to_string(),
                    status: status.to_string(),
                    pinned: pinned.to_string(),
                    created,
                }
            })
            .collect();

        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(size);

            let header = Row::new(
                vec![
                    "Program", "Kind", "Hook", "PID", "Status", "Pinned", "Created",
                ]
                .into_iter()
                .map(|h| Cell::from(Span::styled(h, Style::default().fg(Color::Yellow))))
                .collect::<Vec<_>>(),
            );

            let rows: Vec<Row> = rows_data
                .iter()
                .map(|a| {
                    let status_span = match a.status.as_str() {
                        "online" => Span::styled(
                            "online",
                            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                        ),
                        _ => Span::styled("offline", Style::default().fg(Color::Red)),
                    };
                    let pinned_span = match a.pinned.as_str() {
                        "yes" => Span::styled("yes", Style::default().fg(Color::Green)),
                        "missing" => Span::styled(
                            "missing",
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        ),
                        _ => Span::raw("no"),
                    };
                    Row::new(vec![
                        Cell::from(a.name.clone()),
                        Cell::from(a.kind.clone()),
                        Cell::from(a.hook.clone()),
                        Cell::from(a.pid.clone()),
                        Cell::from(status_span),
                        Cell::from(pinned_span),
                        Cell::from(a.created.clone()),
                    ])
                })
                .collect();

            let table = Table::new(rows)
                .header(header)
                .block(
                    Block::default()
                        .title("eclipta monitor (press 'q' to quit)")
                        .borders(Borders::ALL),
                )
                .widths(&[
                    Constraint::Length(18), // Program
                    Constraint::Length(12), // Kind
                    Constraint::Length(24), // Hook
                    Constraint::Length(8),  // PID
                    Constraint::Length(10), // Status
                    Constraint::Length(8),  // Pinned
                    Constraint::Length(20), // Created
                ])
                .column_spacing(1);

            f.render_widget(table, chunks[0]);
        })?;

        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}
