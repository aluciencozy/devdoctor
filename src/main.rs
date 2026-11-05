use color_eyre::Result;
use crossterm::event::{self, KeyCode};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Row, Table, TableState},
};
use snapshot::SystemSnapshot;
use sysinfo::System;

mod listener;
mod process;
mod runtime;
mod snapshot;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut sys = System::new()?;
    let mut snapshot = SystemSnapshot::new(&mut sys);

    let mut table_state = TableState::default();
    table_state.select_first();

    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| render(frame, &snapshot, &mut table_state))?;

            if let Some(key) = event::read()?.as_key_press_event() {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),

                    KeyCode::Char('j') | KeyCode::Down => {
                        table_state.select_next();
                    }

                    KeyCode::Char('k') | KeyCode::Up => {
                        table_state.select_previous();
                    }

                    KeyCode::Char('g') => table_state.select_first(),
                    KeyCode::Char('G') => table_state.select_last(),

                    KeyCode::Char('r') => {
                        snapshot.refresh(&mut sys);
                    }

                    _ => {}
                }
            }
        }
    })
}

fn render(frame: &mut Frame, snapshot: &SystemSnapshot, table_state: &mut TableState) {
    let layout = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).spacing(1);

    let [top, main] = frame.area().layout(&layout);

    let title = Line::from("DevDoctor | j/k: Navigate | r: Refresh | q: Quit").centered();

    frame.render_widget(title, top);
    render_table(frame, main, snapshot, table_state);
}

fn render_table(
    frame: &mut Frame,
    area: Rect,
    snapshot: &SystemSnapshot,
    table_state: &mut TableState,
) {
    // HashMap iteration is unordered, so sort by PID.
    let mut processes: Vec<_> = snapshot.processes_by_pid.values().collect();

    processes.sort_unstable_by_key(|process| process.pid.as_u32());

    let rows: Vec<Row> = processes
        .into_iter()
        .map(|process| {
            let pid = process.pid.to_string();

            let command = process
                .command
                .iter()
                .map(|arg| arg.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");

            let cwd = process
                .cwd
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "-".to_string());

            let ports = snapshot
                .sockets_by_pid
                .get(&process.pid)
                .map(|sockets| {
                    sockets
                        .iter()
                        .map(|socket| socket.socket.port().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_else(|| "-".to_string());

            Row::new([pid, command, cwd, ports])
        })
        .collect();

    let header = Row::new(["PID", "COMMAND", "CWD", "PORTS"])
        .style(Style::new().bold())
        .bottom_margin(1);

    let widths = [
        Constraint::Length(8),
        Constraint::Percentage(40),
        Constraint::Percentage(35),
        Constraint::Fill(1),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::bordered().title(" Processes "))
        .column_spacing(1)
        .row_highlight_style(Style::new().reversed())
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(table, area, table_state);
}
