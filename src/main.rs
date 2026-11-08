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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Processes,
    Runtimes,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut sys = System::new()?;
    let mut snapshot = SystemSnapshot::new(&mut sys);
    let mut focus = Focus::Processes;

    let mut processes_state = ListState::default();
    processes_state.select(Some(0));

    let mut runtimes_state = ListState::default();
    runtimes_state.select(Some(0));

    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| {
                // get all areas
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

                let processes_title = if focus == Focus::Processes {
                    " Processes [active] "
                } else {
                    " Processes "
                };

                let runtimes_title = if focus == Focus::Runtimes {
                    " Runtimes [active] "
                } else {
                    " Runtimes "
                };

                // agents info (not completed)
                let agents_panel =
                    Paragraph::new("Coming Later").block(Block::bordered().title(" Agents "));

                // runtime info
                let runtime_items = snapshot.runtimes.iter().map(|runtime| {
                    let version = runtime.version.as_deref().unwrap_or("Version unavailable");

                    format!("{}: {}", runtime.name, version)
                });

                let runtimes_panel = List::new(runtime_items)
                    .block(Block::bordered().title(runtimes_title))
                    .highlight_style(Style::new().reversed())
                    .highlight_symbol("▶ ");

                frame.render_stateful_widget(runtimes_panel, runtimes_area, &mut runtimes_state);

                // process info
                let mut processes: Vec<_> = snapshot.processes_by_pid.values().collect();
                processes.sort_unstable_by_key(|process| process.pid.as_u32());

                let process_items = processes.iter().map(|process| {
                    let label = process
                        .command
                        .first()
                        .map(|arg| arg.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "[command unavailable]".to_string());

                    format!("{}  {}", process.pid, label)
                });

                let processes_panel = List::new(process_items)
                    .block(Block::bordered().title(processes_title))
                    .highlight_style(Style::new().reversed())
                    .highlight_symbol("▶ ");

                frame.render_stateful_widget(processes_panel, processes_area, &mut processes_state);

                // main block details info
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
                frame.render_widget(body_panel, body);
                frame.render_widget(
                    "DevDoctor | j/k: Navigate | Tab: Switch panel | r: Refresh | q: Quit",
                    footer_area,
                );
            })?;

            if let Some(key) = crossterm::event::read()?.as_key_press_event() {
                let active_state = match focus {
                    Focus::Processes => &mut processes_state,
                    Focus::Runtimes => &mut runtimes_state,
                };

                match key.code {
                    KeyCode::Char('q') => break,

                    KeyCode::Tab => {
                        focus = match focus {
                            Focus::Processes => Focus::Runtimes,
                            Focus::Runtimes => Focus::Processes,
                        };
                    }

                    KeyCode::Char('j') | KeyCode::Down => active_state.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => active_state.select_previous(),
                    KeyCode::Char('g') | KeyCode::Home => active_state.select_first(),
                    KeyCode::Char('G') | KeyCode::End => active_state.select_last(),
                    KeyCode::PageDown => active_state.scroll_down_by(10),
                    KeyCode::PageUp => active_state.scroll_up_by(10),

                    KeyCode::Char('r') => {
                        snapshot.refresh(&mut sys);
                        processes_state.select_first();
                        runtimes_state.select_first();
                    }

                    _ => {}
                }
            }
        }

        Ok(())
    })
}
