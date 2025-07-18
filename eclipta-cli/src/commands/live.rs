use std::{fs, path::PathBuf, time::Duration};
use serde::Deserialize;
use tokio::time::sleep;

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span},
    widgets::{Block, Borders, Cell, Row, Table},
    Terminal,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

#[derive(Deserialize, Debug)]
pub struct AgentLiveStatus {
    pub id: String,
    pub hostname: String,
    pub uptime_secs: u64,
    pub cpu: Option<f32>,
    pub mem: Option<String>,
}

pub async fn handle_live() {
    let mut stdout = std::io::stdout();
    enable_raw_mode().unwrap();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    loop {
        // Check if user pressed 'q' to quit
        if event::poll(Duration::from_millis(100)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        // Read agent JSON files
        let dir = PathBuf::from("/run/eclipta");
        let mut agents = Vec::new();

        if let Ok(files) = fs::read_dir(&dir) {
            for entry in files.flatten() {
                if entry.file_name().to_string_lossy().starts_with("agent-") {
                    if let Ok(contents) = fs::read_to_string(entry.path()) {
                        if let Ok(agent) = serde_json::from_str::<AgentLiveStatus>(&contents) {
                            agents.push(agent);
                        }
                    }
                }
            }
        }

        // Draw UI
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
                .split(f.size());

            let title = Block::default()
                .title(Span::styled(
                    "ECLIPTA LIVE – Press 'q' to quit",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL);
            f.render_widget(title, chunks[0]);

            let header = ["ID", "Hostname", "Uptime", "CPU%", "Memory"];
            let rows: Vec<Row> = agents
                .iter()
                .map(|a| {
                    Row::new(vec![
                        Cell::from(a.id.clone()),
                        Cell::from(a.hostname.clone()),
                        Cell::from(format!("{}s", a.uptime_secs)),
                        Cell::from(
                            a.cpu
                                .map(|v| format!("{:.1}", v))
                                .unwrap_or_else(|| "--".to_string()),
                        ),
                        Cell::from(a.mem.clone().unwrap_or_else(|| "--".to_string())),
                    ])
                })
                .collect();

            let table = Table::new(rows)
                .header(
                    Row::new(header)
                        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                )
                .block(Block::default().title("Agents").borders(Borders::ALL))
                .widths(&[
                    Constraint::Length(12),
                    Constraint::Length(18),
                    Constraint::Length(10),
                    Constraint::Length(10),
                    Constraint::Length(10),
                ])
                .column_spacing(2);
            f.render_widget(table, chunks[1]);
        }).unwrap();

        sleep(Duration::from_secs(2)).await;
    }

    disable_raw_mode().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
    terminal.show_cursor().unwrap();
}
