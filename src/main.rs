use color_eyre::{Result, eyre::Ok};
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::{Block, Paragraph};
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

    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| {
                let [main_area, footer_area] =
                    Layout::vertical([Constraint::Fill(1), Constraint::Length(1)])
                        .areas(frame.area());

                let [sidebar, body] =
                    Layout::horizontal([Constraint::Percentage(30), Constraint::Fill(1)])
                        .areas(main_area);

                let body_panel = Paragraph::new("Details will go here")
                    .block(Block::bordered().title(" Details "));

                let [processes_area, runtimes_area, agents_area] = Layout::vertical([
                    Constraint::Fill(2),
                    Constraint::Fill(1),
                    Constraint::Fill(1),
                ])
                .areas(sidebar);

                let processes_panel = Paragraph::new("Processes will go here")
                    .block(Block::bordered().title(" Processes "));

                let runtimes_panel = Paragraph::new("Dev tools will go here")
                    .block(Block::bordered().title(" Runtimes "));

                let agents_panel =
                    Paragraph::new("Coming Later").block(Block::bordered().title(" Agents "));

                frame.render_widget(processes_panel, processes_area);
                frame.render_widget(runtimes_panel, runtimes_area);
                frame.render_widget(agents_panel, agents_area);
                frame.render_widget(body_panel, body);
                frame.render_widget("DevDoctor | q: Quit", footer_area);
            })?;

            if let Some(key) = crossterm::event::read()?.as_key_press_event() {
                if key.code == crossterm::event::KeyCode::Char('q') {
                    break;
                }
            }
        }

        Ok(())
    })
}
