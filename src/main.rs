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

fn main() {
    let Ok(mut sys) = System::new() else {
        println!("failed to initialize system");
        return;
    };

    let mut system_snapshot = SystemSnapshot::new(&mut sys);

    println!("{system_snapshot:?}");

    system_snapshot.refresh(&mut sys);

    println!("{system_snapshot:?}");
}
