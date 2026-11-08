use color_eyre::{Result, eyre::Ok};
use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, List, Paragraph, Wrap};

use app::{App, Focus};

mod app;
mod listener;
mod process;
mod runtime;
mod snapshot;
mod ui;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut app = App::new()?;

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

                let processes_border = if app.focus == Focus::Processes {
                    Style::new().cyan()
                } else {
                    Style::new()
                };

                let runtimes_border = if app.focus == Focus::Runtimes {
                    Style::new().cyan()
                } else {
                    Style::new()
                };

                // agents info (not completed)
                let agents_panel =
                    Paragraph::new("Coming Later").block(Block::bordered().title(" Agents "));

                // runtime info
                let runtime_items = app.snapshot.runtimes.iter().map(|runtime| {
                    let version = runtime.version.as_deref().unwrap_or("Version unavailable");

                    format!("{}: {}", runtime.name, version)
                });

                let runtimes_panel = List::new(runtime_items)
                    .block(
                        Block::bordered()
                            .title(" Runtimes ")
                            .border_style(runtimes_border),
                    )
                    .highlight_style(Style::new().reversed())
                    .highlight_symbol("▶ ");

                frame.render_stateful_widget(
                    runtimes_panel,
                    runtimes_area,
                    &mut app.runtimes_state,
                );

                // process info
                let mut processes: Vec<_> = app.snapshot.processes_by_pid.values().collect();
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
                    .block(
                        Block::bordered()
                            .title(" Processes ")
                            .border_style(processes_border),
                    )
                    .highlight_style(Style::new().reversed())
                    .highlight_symbol("▶ ");

                frame.render_stateful_widget(
                    processes_panel,
                    processes_area,
                    &mut app.processes_state,
                );

                // main block details info
                let selected_process = app
                    .processes_state
                    .selected()
                    .and_then(|index| processes.get(index));

                let process_details = match selected_process {
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

                        let listeners = app
                            .snapshot
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

                let selected_runtime = app
                    .runtimes_state
                    .selected()
                    .and_then(|index| app.snapshot.runtimes.get(index));

                let runtime_details = match selected_runtime {
                    Some(runtime) => {
                        let path = runtime
                            .exec_path
                            .as_ref()
                            .map(|path| path.display().to_string())
                            .unwrap_or_else(|| "Not found on PATH".to_string());

                        let version = runtime.version.as_deref().unwrap_or("Version unavailable");

                        format!(
                            "Tool: {}\n\nExecutable:\n{}\n\nVersion:\n{}",
                            runtime.name, path, version
                        )
                    }
                    None => "No runtime selected".to_string(),
                };

                let details = match app.focus {
                    Focus::Processes => process_details,
                    Focus::Runtimes => runtime_details,
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
                let active_state = match app.focus {
                    Focus::Processes => &mut app.processes_state,
                    Focus::Runtimes => &mut app.runtimes_state,
                };

                match key.code {
                    KeyCode::Char('q') => break,

                    KeyCode::Tab => {
                        app.focus = match app.focus {
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
                        app.snapshot.refresh(&mut app.sys);
                        app.processes_state.select_first();
                        app.runtimes_state.select_first();
                    }

                    _ => {}
                }
            }
        }

        Ok(())
    })
}
