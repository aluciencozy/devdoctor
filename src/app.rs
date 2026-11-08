use crate::snapshot::SystemSnapshot;
use crossterm::event::KeyCode;
use ratatui::widgets::ListState;
use sysinfo::{Error, System};

#[derive(Debug)]
pub(crate) struct App {
    sys: System,
    pub(crate) snapshot: SystemSnapshot,
    pub(crate) focus: Focus,
    pub(crate) processes_state: ListState,
    pub(crate) runtimes_state: ListState,
    pub(crate) process_filter: ProcessFilter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Focus {
    Processes,
    Runtimes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProcessFilter {
    All,
    Listening,
}

impl App {
    pub(crate) fn new() -> Result<Self, Error> {
        let mut sys = System::new()?;
        let snapshot = SystemSnapshot::new(&mut sys);
        let focus = Focus::Processes;

        let mut processes_state = ListState::default();
        processes_state.select(Some(0));

        let mut runtimes_state = ListState::default();
        runtimes_state.select(Some(0));

        Ok(App {
            sys,
            snapshot,
            focus,
            processes_state,
            runtimes_state,
            process_filter: ProcessFilter::All,
        })
    }

    pub(crate) fn run(
        &mut self,
        terminal: &mut ratatui::DefaultTerminal,
    ) -> color_eyre::Result<()> {
        loop {
            terminal.draw(|frame| crate::ui::draw(frame, self))?;

            if let Some(key) = crossterm::event::read()?.as_key_press_event() {
                let active_state = match self.focus {
                    Focus::Processes => &mut self.processes_state,
                    Focus::Runtimes => &mut self.runtimes_state,
                };

                match key.code {
                    KeyCode::Char('q') => break,

                    KeyCode::Tab => {
                        self.focus = match self.focus {
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

                    KeyCode::Char('f') if self.focus == Focus::Processes => {
                        self.process_filter = match self.process_filter {
                            ProcessFilter::All => ProcessFilter::Listening,
                            ProcessFilter::Listening => ProcessFilter::All,
                        };

                        self.processes_state.select_first();
                    }

                    KeyCode::Char('r') => {
                        self.snapshot.refresh(&mut self.sys);
                        self.processes_state.select_first();
                        self.runtimes_state.select_first();
                    }

                    _ => {}
                }
            }
        }

        Ok(())
    }
}
