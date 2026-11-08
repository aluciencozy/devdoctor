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
    Status,
    Processes,
    Runtimes,
}

impl Focus {
    fn next(self) -> Self {
        match self {
            Self::Status => Self::Processes,
            Self::Processes => Self::Runtimes,
            Self::Runtimes => Self::Status,
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::Status => Self::Runtimes,
            Self::Processes => Self::Status,
            Self::Runtimes => Self::Processes,
        }
    }
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
        let focus = Focus::Status;

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

            if let Some(key) = crossterm::event::read()?.as_key_press_event()
                && self.handle_key(key.code)
            {
                break;
            }
        }

        Ok(())
    }

    // Returns true only when the user requests to quit.
    fn handle_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('1') => self.focus = Focus::Status,
            KeyCode::Char('2') => self.focus = Focus::Processes,
            KeyCode::Char('3') => self.focus = Focus::Runtimes,
            KeyCode::Char('h') => self.focus = self.focus.previous(),
            KeyCode::Char('l') | KeyCode::Tab => self.focus = self.focus.next(),
            // With two filters, stepping left or right both selects the other one.
            KeyCode::Char('f' | '[' | ']') if self.focus == Focus::Processes => {
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
            _ => {
                // Status has no list: item navigation must not affect hidden selections.
                let active_state = match self.focus {
                    Focus::Status => None,
                    Focus::Processes => Some(&mut self.processes_state),
                    Focus::Runtimes => Some(&mut self.runtimes_state),
                };
                if let Some(state) = active_state {
                    match code {
                        KeyCode::Char('j') | KeyCode::Down => state.select_next(),
                        KeyCode::Char('k') | KeyCode::Up => state.select_previous(),
                        KeyCode::Char('g') | KeyCode::Home => state.select_first(),
                        KeyCode::Char('G') | KeyCode::End => state.select_last(),
                        KeyCode::PageDown => state.scroll_down_by(10),
                        KeyCode::PageUp => state.scroll_up_by(10),
                        _ => {}
                    }
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    // No process collection or external version commands are needed for these tests.
    fn app() -> App {
        App {
            sys: System::new().unwrap(),
            snapshot: SystemSnapshot {
                runtimes: Vec::new(),
                processes_by_pid: Default::default(),
                sockets_by_pid: Default::default(),
            },
            focus: Focus::Status,
            processes_state: ListState::default().with_selected(Some(0)),
            runtimes_state: ListState::default().with_selected(Some(0)),
            process_filter: ProcessFilter::All,
        }
    }

    #[test]
    fn numbers_jump_directly_without_resetting_selections() {
        let mut app = app();
        app.processes_state.select(Some(5));
        app.runtimes_state.select(Some(2));

        for (key, focus) in [
            ('3', Focus::Runtimes),
            ('1', Focus::Status),
            ('2', Focus::Processes),
        ] {
            assert!(!app.handle_key(KeyCode::Char(key)));
            assert_eq!(app.focus, focus);
            assert_eq!(app.processes_state.selected(), Some(5));
            assert_eq!(app.runtimes_state.selected(), Some(2));
        }
        app.handle_key(KeyCode::Char('4'));
        assert_eq!(app.focus, Focus::Processes);
        assert!(app.handle_key(KeyCode::Char('q')));
    }

    #[test]
    fn section_navigation_wraps_in_both_directions() {
        let mut app = app();
        for key in [KeyCode::Char('l'), KeyCode::Tab] {
            for focus in [Focus::Processes, Focus::Runtimes, Focus::Status] {
                app.handle_key(key);
                assert_eq!(app.focus, focus);
            }
        }
        for focus in [Focus::Runtimes, Focus::Processes, Focus::Status] {
            app.handle_key(KeyCode::Char('h'));
            assert_eq!(app.focus, focus);
        }
    }

    #[test]
    fn status_ignores_list_navigation_and_filtering() {
        let mut app = app();
        app.processes_state.select(Some(5));
        app.runtimes_state.select(Some(2));
        for code in [
            KeyCode::Char('j'),
            KeyCode::Char('k'),
            KeyCode::Char('g'),
            KeyCode::Char('G'),
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::Home,
            KeyCode::End,
            KeyCode::PageDown,
            KeyCode::PageUp,
            KeyCode::Char('f'),
            KeyCode::Char('['),
            KeyCode::Char(']'),
        ] {
            app.handle_key(code);
            assert_eq!(app.focus, Focus::Status);
            assert_eq!(app.processes_state.selected(), Some(5));
            assert_eq!(app.runtimes_state.selected(), Some(2));
            assert_eq!(app.process_filter, ProcessFilter::All);
        }
    }

    #[test]
    fn list_navigation_and_filtering_still_follow_focus() {
        let mut app = app();
        app.handle_key(KeyCode::Char('2'));
        app.handle_key(KeyCode::Char('j'));
        assert_eq!(app.processes_state.selected(), Some(1));
        assert_eq!(app.runtimes_state.selected(), Some(0));

        app.handle_key(KeyCode::Char('3'));
        app.handle_key(KeyCode::Char('j'));
        app.handle_key(KeyCode::Char('f'));
        assert_eq!(app.runtimes_state.selected(), Some(1));
        assert_eq!(app.process_filter, ProcessFilter::All);

        app.handle_key(KeyCode::Char('2'));
        app.handle_key(KeyCode::Char('f'));
        assert_eq!(app.processes_state.selected(), Some(0));
        assert_eq!(app.runtimes_state.selected(), Some(1));
        assert_eq!(app.process_filter, ProcessFilter::Listening);
    }

    #[test]
    fn bracket_keys_wrap_filters_only_in_processes() {
        for key in ['[', ']'] {
            let mut app = app();
            app.focus = Focus::Processes;
            for expected in [ProcessFilter::Listening, ProcessFilter::All] {
                app.processes_state.select(Some(5));
                app.handle_key(KeyCode::Char(key));
                assert_eq!(app.process_filter, expected);
                assert_eq!(app.processes_state.selected(), Some(0));
            }
            for focus in [Focus::Status, Focus::Runtimes] {
                app.focus = focus;
                app.processes_state.select(Some(5));
                app.handle_key(KeyCode::Char(key));
                assert_eq!(app.process_filter, ProcessFilter::All);
                assert_eq!(app.processes_state.selected(), Some(5));
            }
        }
    }

    #[test]
    fn process_rows_keep_a_gap_after_short_and_long_pids() {
        for pid in [42, 1_234_567, i32::MAX as u32] {
            let mut app = app();
            app.focus = Focus::Processes;
            let process = crate::process::ProcessInfo {
                pid: sysinfo::Pid::from_u32(pid),
                command: vec!["node".into()],
                cwd: None,
            };
            app.snapshot.processes_by_pid.insert(process.pid, process);
            let mut terminal = Terminal::new(TestBackend::new(140, 40)).unwrap();
            terminal
                .draw(|frame| crate::ui::draw(frame, &mut app))
                .unwrap();
            let buffer = terminal.backend().buffer();
            let screen: String = (0..40)
                .flat_map(|y| (0..140).map(move |x| buffer[(x, y)].symbol()))
                .collect();
            assert!(screen.contains(&format!("{:<7}  node", pid.to_string())));
        }
    }

    #[test]
    fn numbered_sections_render_the_corresponding_main_view() {
        let mut app = app();
        for (key, heading) in [
            ('1', "Version"),
            ('2', "Process details"),
            ('3', "Runtime details"),
        ] {
            app.handle_key(KeyCode::Char(key));
            let mut terminal = Terminal::new(TestBackend::new(140, 40)).unwrap();
            terminal
                .draw(|frame| crate::ui::draw(frame, &mut app))
                .unwrap();
            let buffer = terminal.backend().buffer();
            let screen: String = (0..40)
                .flat_map(|y| (0..140).map(move |x| buffer[(x, y)].symbol()))
                .collect();
            for title in ["[1] Status", "[2] Processes", "[3] Runtimes", heading] {
                assert!(screen.contains(title), "Missing {title}");
            }
            assert!(!screen.contains("[4]"));
            if key == '1' {
                assert!(screen.contains("|  _ \\"));
                assert!(screen.contains("0 processes"));
                assert_eq!(buffer[(0, 0)].fg, ratatui::style::Color::Rgb(245, 194, 231));
            }
        }
    }

    #[test]
    fn all_views_render_on_small_terminals() {
        for (width, height) in [(1, 1), (20, 8), (80, 24)] {
            for focus in [Focus::Status, Focus::Processes, Focus::Runtimes] {
                let mut app = app();
                app.focus = focus;
                let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
                terminal
                    .draw(|frame| crate::ui::draw(frame, &mut app))
                    .unwrap();
            }
        }
    }
}
