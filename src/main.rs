use color_eyre::{Result, eyre::Ok};
use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, List, ListState, Paragraph, Wrap};
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

    let mut processes_state = ListState::default();
    processes_state.select(Some(0));

    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| {
                let [main_area, footer_area] =
                    Layout::vertical([Constraint::Fill(1), Constraint::Length(1)])
                        .areas(frame.area());

                let [sidebar, body] =
                    Layout::horizontal([Constraint::Percentage(30), Constraint::Fill(1)])
                        .areas(main_area);

                let [processes_area, runtimes_area, agents_area] = Layout::vertical([
                    Constraint::Fill(2),
                    Constraint::Fill(1),
                    Constraint::Fill(1),
                ])
                .areas(sidebar);

                let runtimes_panel = Paragraph::new("Dev tools will go here")
                    .block(Block::bordered().title(" Runtimes "));

                let agents_panel =
                    Paragraph::new("Coming Later").block(Block::bordered().title(" Agents "));

                let mut processes: Vec<_> = snapshot.processes_by_pid.values().collect();
                processes.sort_unstable_by_key(|process| process.pid.as_u32());

                let items = processes
                    .iter()
                    .map(|process| format!("PID {}", process.pid));

                let processes_panel = List::new(items)
                    .block(Block::bordered().title(" Processes "))
                    .highlight_style(Style::new().reversed())
                    .highlight_symbol("▶ ");

                frame.render_stateful_widget(processes_panel, processes_area, &mut processes_state);

                let selected_process = processes_state
                    .selected()
                    .and_then(|index| processes.get(index));

                let details = match selected_process {
                    Some(process) => {
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
                            .unwrap_or_else(|| "Unavailable".to_string());

                        let listeners = snapshot
                            .sockets_by_pid
                            .get(&process.pid)
                            .map(|sockets| {
                                sockets
                                    .iter()
                                    .map(|socket| socket.socket.to_string())
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            })
                            .filter(|text| !text.is_empty())
                            .unwrap_or_else(|| "None reported".to_string());

                        format!(
                            "PID: {}\n\nWorking directory:\n{}\n\nTCP \
                             listeners:\n{}\n\nCommand:\n{}",
                            process.pid, cwd, listeners, command
                        )
                    }
                    None => "No process selected".to_string(),
                };

                let body_panel = Paragraph::new(details)
                    .block(Block::bordered().title(" Details "))
                    .wrap(Wrap { trim: false });

                frame.render_widget(agents_panel, agents_area);
                frame.render_widget("DevDoctor | q: Quit", footer_area);
                frame.render_widget(runtimes_panel, runtimes_area);
                frame.render_widget(body_panel, body);
            })?;

            if let Some(key) = crossterm::event::read()?.as_key_press_event() {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') | KeyCode::Down => processes_state.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => processes_state.select_previous(),
                    _ => {}
                }
            }
        }

        Ok(())
    })
}
