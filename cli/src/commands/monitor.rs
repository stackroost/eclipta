use chrono::{DateTime, Utc};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use serde::Deserialize;
use std::{fs, io, path::PathBuf, time::Duration};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table},
    Terminal,
};

#[derive(Deserialize)]
struct Agent {
    id: String,
    hostname: String,
    kernel: String,
    version: String,
    last_seen: String,
    uptime_secs: u64,
    cpu_load: [f32; 3],
    mem_used_mb: u64,
    mem_total_mb: u64,
    process_count: u32,
    disk_used_mb: u64,
    disk_total_mb: u64,
    net_rx_kb: u64,
    net_tx_kb: u64,
    tcp_connections: u32,
    alert: bool,
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

        let mut agents = Vec::new();
        if let Ok(files) = fs::read_dir("/run/eclipta") {
            for entry in files.flatten() {
                if let Ok(txt) = fs::read_to_string(entry.path()) {
                    if let Ok(agent) = serde_json::from_str::<Agent>(&txt) {
                        agents.push(agent);
                    }
                }
            }
        }

        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(size);

            let header = Row::new(
                vec![
                    "ID", "Host", "Uptime", "CPU", "Mem", "Disk", "Net", "TCP", "Proc", "Alert",
                    "Seen",
                ]
                .into_iter()
                .map(|h| Cell::from(Span::styled(h, Style::default().fg(Color::Yellow))))
                .collect::<Vec<_>>(),
            );

            let rows: Vec<Row> = agents
                .iter()
                .map(|a| {
                    let cpu = format!(
                        "{:.1}/{:.1}/{:.1}",
                        a.cpu_load[0], a.cpu_load[1], a.cpu_load[2]
                    );
                    let mem = format!("{} / {}", a.mem_used_mb, a.mem_total_mb);
                    let disk = format!("{} / {}", a.disk_used_mb, a.disk_total_mb);
                    let net = format!(
                        "{:.1}↓ / {:.1}↑",
                        a.net_rx_kb as f64 / 1024.0,
                        a.net_tx_kb as f64 / 1024.0
                    );
                    let seen = DateTime::parse_from_rfc3339(&a.last_seen)
                        .map(|dt| dt.with_timezone(&Utc).format("%H:%M:%S").to_string())
                        .unwrap_or_else(|_| "-".into());
                    let alert = if a.alert {
                        Span::styled(
                            "⚠️",
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        )
                    } else {
                        Span::raw("✅")
                    };

                    Row::new(vec![
                        Cell::from(a.id.clone()),
                        Cell::from(a.hostname.clone()),
                        Cell::from(format!("{}s", a.uptime_secs)),
                        Cell::from(cpu),
                        Cell::from(mem),
                        Cell::from(disk),
                        Cell::from(net),
                        Cell::from(format!("{}", a.tcp_connections)),
                        Cell::from(format!("{}", a.process_count)),
                        Cell::from(alert),
                        Cell::from(seen),
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
                    Constraint::Length(10), // ID
                    Constraint::Length(15), // Hostname
                    Constraint::Length(8),  // Uptime
                    Constraint::Length(12), // CPU
                    Constraint::Length(14), // Mem
                    Constraint::Length(14), // Disk
                    Constraint::Length(14), // Net
                    Constraint::Length(5),  // TCP
                    Constraint::Length(6),  // Proc
                    Constraint::Length(6),  // Alert
                    Constraint::Length(8),  // Last seen
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
