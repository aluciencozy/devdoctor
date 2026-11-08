use crate::snapshot::SystemSnapshot;
use ratatui::widgets::ListState;
use sysinfo::{Error, System};

#[derive(Debug)]
pub struct App {
    pub(crate) sys: System,
    pub(crate) snapshot: SystemSnapshot,
    pub(crate) focus: Focus,
    pub(crate) processes_state: ListState,
    pub(crate) runtimes_state: ListState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Processes,
    Runtimes,
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
        })
    }
}
