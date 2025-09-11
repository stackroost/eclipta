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
use crate::utils::db::ensure_db_ready;
use crate::db::programs::list_programs;
use serde_json::Value;
use std::collections::HashMap;
use tokio::process::Command as TokioCommand;

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

#[derive(Debug, Clone)]
struct LinkInfo {
    prog_id: u32,
    pid: Option<u32>,
    attach_type: String,
    target: Option<String>,
    hook: Option<String>,
}

async fn get_live_bpf_indices() -> Result<(HashMap<String, u32>, Vec<LinkInfo>), std::io::Error> {
    let prog_out = TokioCommand::new("bpftool").args(["prog", "list", "-j"]).output().await;
    let link_out = TokioCommand::new("bpftool").args(["link", "list", "-j"]).output().await;

    let mut name_to_id: HashMap<String, u32> = HashMap::new();
    let mut links: Vec<LinkInfo> = Vec::new();

    if let Ok(o) = prog_out {
        if o.status.success() {
            if let Ok(v) = serde_json::from_slice::<Value>(&o.stdout) {
                if let Some(arr) = v.as_array() {
                    for p in arr {
                        let id = p.get("id").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
                        if id == 0 { continue; }
                        if let Some(name) = p.get("name").and_then(|x| x.as_str()) {
                            name_to_id.insert(name.to_string(), id);
                        } else if let Some(tag) = p.get("tag").and_then(|x| x.as_str()) {
                            name_to_id.insert(tag.to_string(), id);
                        }
                    }
                }
            }
        }
    }

    if let Ok(o) = link_out {
        if o.status.success() {
            if let Ok(v) = serde_json::from_slice::<Value>(&o.stdout) {
                if let Some(arr) = v.as_array() {
                    for l in arr {
                        let prog_id = l.get("prog_id").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
                        if prog_id == 0 { continue; }
                        let attach_type = l.get("type").and_then(|x| x.as_str()).unwrap_or("").to_string();
                        let target = l.get("target_name").and_then(|x| x.as_str()).map(|s| s.to_string());
                        let hook = l.get("tp_name").and_then(|x| x.as_str()).map(|s| s.to_string());
                        let mut pid: Option<u32> = None;
                        if let Some(pid_val) = l.get("pid").and_then(|x| x.as_u64()) {
                            pid = Some(pid_val as u32);
                        } else if let Some(pids) = l.get("pids").and_then(|x| x.as_array()) {
                            for p in pids {
                                if let Some(pv) = p.get("pid").and_then(|x| x.as_u64()) {
                                    pid = Some(pv as u32);
                                    break;
                                }
                            }
                        }
                        links.push(LinkInfo { prog_id, pid, attach_type, target, hook });
                    }
                }
            }
        }
    }

    Ok((name_to_id, links))
}

fn live_hook_for(
    prog_index: &HashMap<String, u32>,
    link_index: &Vec<LinkInfo>,
    program_name: &str,
    pid: u32,
) -> Option<String> {
    let prog_id = prog_index.get(program_name).copied()?;
    let mut candidates: Vec<&LinkInfo> = link_index.iter().filter(|l| l.prog_id == prog_id).collect();
    if candidates.is_empty() { return None; }
    if let Some(first_pid_match) = candidates.iter().find(|l| l.pid == Some(pid)) {
        return Some(render_hook(first_pid_match));
    }
    Some(render_hook(candidates[0]))
}

fn render_hook(l: &LinkInfo) -> String {
    let tgt = l.hook.clone().or_else(|| l.target.clone()).unwrap_or_default();
    if tgt.is_empty() { l.attach_type.clone() } else { format!("{}:{}", l.attach_type, tgt) }
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

        let (prog_index, link_index) = match get_live_bpf_indices().await {
            Ok(v) => v,
            Err(_) => (HashMap::new(), Vec::new()),
        };
        let mut rows_data: Vec<AttachmentRow> = st
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
                let live_hook = live_hook_for(&prog_index, &link_index, &r.name, r.pid)
                    .unwrap_or(hook);

                let status = if proc_alive(r.pid) { "online" } else { "offline" };
                let created = DateTime::from_timestamp(r.created_at, 0)
                    .unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
                    .with_timezone(&Utc)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string();
                AttachmentRow {
                    name: r.name.clone(),
                    kind: r.kind.clone(),
                    hook: live_hook,
                    pid: r.pid.to_string(),
                    status: status.to_string(),
                    pinned: pinned.to_string(),
                    created,
                }
            })
            .collect();

        if let Ok(pool) = ensure_db_ready().await {
            if let Ok(programs) = list_programs(&pool).await {
                let state_names: std::collections::HashSet<String> = st.attachments.iter().map(|a| a.name.clone()).collect();
                let any_attached = link_index.iter().next();
                for p in programs {
                    if state_names.contains(&p.title) { continue; }
                    let (pid_str, hook_str, status_str) = if let Some(l) = any_attached {
                        let pid_str = l.pid.map(|x| x.to_string()).unwrap_or("-".to_string());
                        let hook_str = render_hook(l);
                        (pid_str, hook_str, "attached".to_string())
                    } else {
                        ("-".to_string(), "-".to_string(), "detached".to_string())
                    };
                    rows_data.push(AttachmentRow {
                        name: p.title,
                        kind: p.status,
                        hook: hook_str,
                        pid: pid_str,
                        status: status_str,
                        pinned: "n/a".to_string(),
                        created: "-".to_string(),
                    });
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
                    Constraint::Length(18),
                    Constraint::Length(12),
                    Constraint::Length(24),
                    Constraint::Length(8),
                    Constraint::Length(10),
                    Constraint::Length(8),
                    Constraint::Length(20),
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
