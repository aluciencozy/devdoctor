use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use snapshot::SystemSnapshot;
use sysinfo::System;

mod listener;
mod process;
mod runtime;
mod snapshot;

#[derive(Default, Debug)]
struct App {
    exit: bool,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        todo!()
    }

    fn handle_events(&mut self) -> io::Result<()> {
        todo!()
    }
}

fn main() -> io::Result<()> {
    let Ok(mut sys) = System::new() else {
        panic!("failed to initialize system");
    };

    let mut system_snapshot = SystemSnapshot::new(&mut sys);

    ratatui::run(|terminal| App::default().run(terminal));
    Ok(())
}
