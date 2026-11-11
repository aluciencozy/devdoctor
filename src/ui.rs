use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, List, ListItem, Padding, Paragraph, Wrap};

use crate::app::{App, Focus, ProcessFilter};
use crate::snapshot::SystemSnapshot;

// catppuccin palette
const BACKGROUND: Color = Color::Rgb(30, 30, 46);
const TEXT: Color = Color::Rgb(205, 214, 244);
const MUTED: Color = Color::Rgb(108, 112, 134);
const BORDER: Color = Color::Rgb(69, 71, 90);
const PINK: Color = Color::Rgb(245, 194, 231);
const GREEN: Color = Color::Rgb(166, 227, 161);
const BLUE: Color = Color::Rgb(137, 180, 250);
const CYAN: Color = Color::Rgb(137, 220, 235);
const SELECTION: Color = Color::Rgb(69, 71, 90);
const INACTIVE_SELECTION: Color = Color::Rgb(38, 38, 55);

const LOGO: &str = r" ____             ____             _
|  _ \  _____   _|  _ \  ___   ___| |_ ___  _ __
| | | |/ _ \ \ / / | | |/ _ \ / __| __/ _ \| '__|
| |_| |  __/\ V /| |_| | (_) | (__| || (_) | |
|____/ \___| \_/ |____/ \___/ \___|\__\___/|_|";

pub(crate) fn draw(frame: &mut Frame, app: &mut App) {
    frame.render_widget(
        Block::default().style(Style::new().fg(TEXT).bg(BACKGROUND)),
        frame.area(),
    );

    let editing_search = app.is_querying && app.focus != Focus::Status;
    let [main_area, search_area, footer_area] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(u16::from(editing_search)),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let [sidebar, right] = Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)])
        .spacing(1)
        .areas(main_area);

    let [status_area, processes_area, runtimes_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(2),
        Constraint::Fill(1),
    ])
    .spacing(1)
    .areas(sidebar);

    let [body, context_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(5)])
        .spacing(1)
        .areas(right);

    let processes_focused = app.focus == Focus::Processes;
    let runtimes_focused = app.focus == Focus::Runtimes;

    let status = Paragraph::new(Line::from(vec![
        Span::styled("● ", Style::new().fg(GREEN)),
        Span::styled("devdoctor", Style::new().fg(TEXT).bold()),
        Span::styled(" · local", Style::new().fg(MUTED)),
    ]))
    .block(panel(Line::from("[1] Status"), app.focus == Focus::Status));
    frame.render_widget(status, status_area);

    // runtime info
    let runtime_query = app.runtime_query.to_lowercase();
    let runtimes: Vec<_> = app
        .snapshot
        .runtimes
        .iter()
        .filter(|runtime| runtime.name.to_lowercase().contains(&runtime_query))
        .collect();

    let runtime_items = runtimes.iter().map(|runtime| {
        let summary = match &runtime.candidates {
            Ok(candidates) if candidates.is_empty() => "Not found".to_string(),
            Ok(candidates) => format!("{} on PATH", candidates.len()),
            Err(_) => "Lookup failed".to_string(),
        };

        ListItem::new(Line::from(vec![
            Span::styled(runtime.name.clone(), Style::new().fg(GREEN)),
            Span::raw("  "),
            Span::styled(summary, Style::new().fg(MUTED)),
        ]))
    });

    let runtime_position = position(app.runtimes_state.selected(), runtimes.len());
    let runtime_search = search_title(
        &app.runtime_query,
        editing_search && runtimes_focused,
        usize::from(runtimes_area.width.saturating_sub(3)).saturating_sub(runtime_position.width()),
    );
    let runtimes_panel = List::new(runtime_items)
        .block(
            panel(Line::from("[3] Runtimes"), runtimes_focused)
                .title_bottom(runtime_search)
                .title_bottom(runtime_position.right_aligned()),
        )
        .highlight_style(selection_style(runtimes_focused))
        .highlight_symbol(" ");

    frame.render_stateful_widget(runtimes_panel, runtimes_area, &mut app.runtimes_state);
    if runtimes.is_empty() {
        frame.render_widget(
            Paragraph::new("No matching runtimes").style(Style::new().fg(MUTED)),
            Block::bordered().inner(runtimes_area),
        );
    }

    // process info
    let process_query = app.process_query.to_lowercase();
    let mut processes: Vec<_> = app
        .snapshot
        .processes_by_pid
        .values()
        .filter(|process| match app.process_filter {
            ProcessFilter::All => true,
            ProcessFilter::Listening => app.snapshot.sockets_by_pid.contains_key(&process.pid),
        })
        .filter(|process| {
            process_query.is_empty() || process.pid.to_string().contains(&process_query) || {
                let command = process
                    .command
                    .iter()
                    .map(|os_string| os_string.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(" ");
                command.to_lowercase().contains(&process_query)
            }
        })
        .collect();
    processes.sort_unstable_by_key(|process| process.pid.as_u32());

    let process_items = processes.iter().map(|process| {
        let label = process
            .command
            .first()
            .map(|arg| arg.to_string_lossy().into_owned())
            .unwrap_or_else(|| "[command unavailable]".to_string());

        ListItem::new(Line::from(vec![
            Span::styled(
                format!("{:<7}  ", process.pid.to_string()),
                Style::new().fg(GREEN),
            ),
            Span::raw(label),
        ]))
    });

    let filter_style = |filter| {
        if app.process_filter == filter {
            Style::new().fg(TEXT).bold().underlined()
        } else {
            Style::new().fg(MUTED)
        }
    };
    let processes_title = if sidebar.width >= 34 {
        Line::from(vec![
            Span::raw("[2] Processes "),
            Span::styled("· ", Style::new().fg(MUTED)),
            Span::styled("All", filter_style(ProcessFilter::All)),
            Span::styled(" / ", Style::new().fg(MUTED)),
            Span::styled("Listening", filter_style(ProcessFilter::Listening)),
        ])
    } else if sidebar.width >= 23 {
        let filter_label = match app.process_filter {
            ProcessFilter::All => "All",
            ProcessFilter::Listening => "TCP",
        };
        Line::from(vec![
            Span::raw("[2] Processes · "),
            Span::styled(filter_label, filter_style(app.process_filter)),
        ])
    } else {
        Line::from("[2] Processes")
    };

    let process_position = position(app.processes_state.selected(), processes.len());
    let process_search = search_title(
        &app.process_query,
        editing_search && processes_focused,
        usize::from(processes_area.width.saturating_sub(3))
            .saturating_sub(process_position.width()),
    );
    let processes_panel = List::new(process_items)
        .block(
            panel(processes_title, processes_focused)
                .title_bottom(process_search)
                .title_bottom(process_position.right_aligned()),
        )
        .highlight_style(selection_style(processes_focused))
        .highlight_symbol(" ");

    frame.render_stateful_widget(processes_panel, processes_area, &mut app.processes_state);
    if processes.is_empty() {
        frame.render_widget(
            Paragraph::new("No matching processes").style(Style::new().fg(MUTED)),
            Block::bordered().inner(processes_area),
        );
    }

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

            let listeners = match &app.snapshot.listener_error {
                Some(error) => format!("Listener discovery failed: {error}"),
                None => app
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
                    .unwrap_or_else(|| "None reported".to_string()),
            };

            detail_sections([
                ("PROCESS ID", process.pid.to_string()),
                ("WORKING DIRECTORY", cwd),
                ("TCP LISTENERS", listeners),
                ("COMMAND", command),
            ])
        }
        None => Text::from("No process selected"),
    };

    let selected_runtime = app
        .runtimes_state
        .selected()
        .and_then(|index| runtimes.get(index));

    let runtime_details = match selected_runtime {
        Some(runtime) => {
            let executables = match &runtime.candidates {
                Ok(candidates) if candidates.is_empty() => {
                    "No executable found on PATH".to_string()
                }
                Ok(candidates) => candidates
                    .iter()
                    .enumerate()
                    .map(|(index, candidate)| {
                        let precedence = if index == 0 {
                            "first PATH match"
                        } else {
                            "later PATH match"
                        };

                        let version = match &candidate.version {
                            Ok(version) => version.clone(),
                            Err(error) => format!("Version check failed: {error}"),
                        };

                        format!(
                            "{}. {} ({})\n{}",
                            index + 1,
                            candidate.path.display(),
                            precedence,
                            version,
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n"),
                Err(error) => format!("Executable lookup failed: {error}"),
            };

            detail_sections([
                ("TOOL", runtime.name.clone()),
                ("EXECUTABLES · PATH ORDER", executables),
            ])
        }
        None => Text::from("No runtime selected"),
    };

    let details = match app.focus {
        Focus::Status => status_details(&app.snapshot, body.width.saturating_sub(4)),
        Focus::Processes => process_details,
        Focus::Runtimes => runtime_details,
    };

    let details_title = match app.focus {
        Focus::Status => "Status",
        Focus::Processes => "Process details",
        Focus::Runtimes => "Runtime details",
    };

    let details_block = panel(Line::from(details_title), false).padding(Padding::horizontal(1));
    let details_area = details_block.inner(body);
    let body_panel = Paragraph::new(details).wrap(Wrap { trim: false });

    let max_scroll = if details_area.is_empty() {
        0
    } else {
        body_panel
            .line_count(details_area.width)
            .saturating_sub(usize::from(details_area.height))
    };
    let max_scroll = u16::try_from(max_scroll).unwrap_or(u16::MAX);

    app.details_scroll = app.details_scroll.min(max_scroll);
    let scroll_hint = if max_scroll > 0 {
        format!(
            " J/K: scroll · {}/{} ",
            app.details_scroll + 1,
            u32::from(max_scroll) + 1
        )
    } else {
        " J/K: scroll ".to_string()
    };

    frame.render_widget(
        details_block
            .title_bottom(Line::styled(scroll_hint, Style::new().fg(MUTED)).right_aligned()),
        body,
    );
    frame.render_widget(body_panel.scroll((app.details_scroll, 0)), details_area);

    let context = match app.focus {
        Focus::Status => Text::from(vec![
            Line::styled(
                "Find development services left running on your machine",
                Style::new().fg(CYAN),
            ),
            Line::from(""),
            Line::from("1/2/3: jump to a section · h/l or Tab: switch sections"),
        ]),
        Focus::Processes => {
            let filter = match app.process_filter {
                ProcessFilter::All => "all processes",
                ProcessFilter::Listening => "reported TCP listeners",
            };
            let listener_failure = app
                .snapshot
                .listener_error
                .as_ref()
                .filter(|_| app.process_filter == ProcessFilter::Listening)
                .map(|error| {
                    Line::styled(
                        format!("Listener discovery failed: {error}"),
                        Style::new().fg(PINK),
                    )
                });
            Text::from(vec![
                Line::styled(
                    format!(
                        "{} shown · {} collected · {}",
                        processes.len(),
                        app.snapshot.processes_by_pid.len(),
                        filter
                    ),
                    Style::new().fg(CYAN),
                ),
                query_status(&app.process_query),
                listener_failure.unwrap_or_else(|| {
                    Line::from("Snapshot data · r: refresh · [: previous filter · ]: next filter")
                }),
            ])
        }
        Focus::Runtimes => Text::from(vec![
            Line::styled(
                format!(
                    "{} shown · {} configured tools",
                    runtimes.len(),
                    app.snapshot.runtimes.len()
                ),
                Style::new().fg(CYAN),
            ),
            query_status(&app.runtime_query),
            Line::from("Snapshot data · press r to check paths and versions again"),
        ]),
    };
    frame.render_widget(
        Paragraph::new(context)
            .block(panel(Line::from("Context"), false))
            .wrap(Wrap { trim: false }),
        context_area,
    );
    if editing_search {
        let query = match app.focus {
            Focus::Processes => &app.process_query,
            Focus::Runtimes => &app.runtime_query,
            Focus::Status => unreachable!(),
        };
        draw_search_editor(frame, search_area, footer_area, app.focus, query);
    } else {
        draw_footer(frame, footer_area, app.focus);
    }
}

fn search_title(query: &str, editing: bool, width: usize) -> Line<'static> {
    if (query.is_empty() && !editing) || width < 4 {
        return Line::default();
    }

    let label = format!(" /{query} ");
    let label = if Line::from(label.as_str()).width() <= width {
        label
    } else {
        // Measure terminal columns, not UTF-8 bytes, and leave room for the counter.
        let mut shortened = String::from(" /");
        for c in query.chars() {
            if Line::from(format!("{shortened}{c}… ")).width() > width {
                break;
            }
            shortened.push(c);
        }
        shortened.push_str("… ");
        shortened
    };

    Line::styled(label, Style::new().fg(if editing { PINK } else { CYAN }))
}

fn query_status(query: &str) -> Line<'_> {
    if query.is_empty() {
        Line::styled("Search: none · / to search", Style::new().fg(MUTED))
    } else {
        Line::from(vec![
            Span::styled("Search: /", Style::new().fg(MUTED)),
            Span::styled(query, Style::new().fg(TEXT)),
        ])
    }
}

fn draw_search_editor(frame: &mut Frame, area: Rect, footer: Rect, focus: Focus, query: &str) {
    frame.render_widget(
        Line::from(vec![
            Span::styled("Type to filter", Style::new().fg(PINK).bold()),
            Span::styled(
                " │ Enter: Keep │ Esc: Clear │ Backspace: Delete",
                Style::new().fg(BLUE),
            ),
        ]),
        footer,
    );
    if area.is_empty() {
        return;
    }

    frame.render_widget(Block::default().style(Style::new().bg(SELECTION)), area);
    let label = match focus {
        Focus::Processes => "Search processes / ",
        Focus::Runtimes => "Search runtimes / ",
        Focus::Status => return,
    };
    // Keep at least one cell for input, even on a very narrow terminal.
    let label_width = u16::try_from(label.len())
        .unwrap_or(u16::MAX)
        .min(area.width.saturating_sub(1));
    let [label_area, input_area] =
        Layout::horizontal([Constraint::Length(label_width), Constraint::Fill(1)]).areas(area);
    frame.render_widget(
        Line::styled(label, Style::new().fg(PINK).bold()),
        label_area,
    );

    let query_width = Line::from(query).width();
    let scroll =
        u16::try_from(query_width.saturating_sub(usize::from(input_area.width.saturating_sub(1))))
            .unwrap_or(u16::MAX);
    frame.render_widget(
        Paragraph::new(query)
            .style(Style::new().fg(TEXT))
            .scroll((0, scroll)),
        input_area,
    );
    let cursor_column = query_width
        .saturating_sub(usize::from(scroll))
        .min(usize::from(input_area.width.saturating_sub(1)));
    frame.set_cursor_position((input_area.x + cursor_column as u16, input_area.y));
}

fn panel(title: Line<'_>, focused: bool) -> Block<'_> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(if focused { PINK } else { BORDER }))
        .title_style(Style::new().fg(if focused { PINK } else { MUTED }).bold())
        .title(title)
}

fn selection_style(focused: bool) -> Style {
    Style::new().bg(if focused {
        SELECTION
    } else {
        INACTIVE_SELECTION
    })
}

fn position(selected: Option<usize>, count: usize) -> Line<'static> {
    let current = selected.map_or(0, |index| index.saturating_add(1).min(count));
    Line::styled(format!(" {current} of {count} "), Style::new().fg(MUTED))
}

fn detail_sections(sections: impl IntoIterator<Item = (&'static str, String)>) -> Text<'static> {
    let mut text = Text::default();
    for (label, value) in sections {
        if !text.lines.is_empty() {
            text.lines.push(Line::from(""));
        }
        text.lines
            .push(Line::styled(label, Style::new().fg(BLUE).bold()));

        // value could contain multiple lines (ex: listening addresses)
        // converting to Text splits its newlines into separate Lines
        // then extend appends all of them
        text.lines.extend(Text::from(value).lines);
    }
    text
}

fn status_details(snapshot: &SystemSnapshot, width: u16) -> Text<'static> {
    let logo_width = LOGO.lines().map(str::len).max().unwrap_or(0);
    let logo = if usize::from(width) >= logo_width {
        LOGO
    } else {
        "[ DevDoctor ]"
    };

    let mut lines = vec![Line::from("")];
    lines.extend(
        logo.lines()
            .map(|line| Line::styled(line, Style::new().fg(PINK).bold())),
    );
    lines.extend([
        Line::from(""),
        Line::styled(
            "A checkup for your development environment",
            Style::new().fg(CYAN),
        ),
        Line::from(""),
    ]);

    let listener_count: usize = snapshot.sockets_by_pid.values().map(Vec::len).sum();
    let listener_summary = match &snapshot.listener_error {
        Some(error) => format!("listener discovery failed: {error}"),
        None => format!("{listener_count} TCP listeners"),
    };
    let tools_found = snapshot
        .runtimes
        .iter()
        .filter(|runtime| {
            runtime
                .candidates
                .as_ref()
                .is_ok_and(|candidates| !candidates.is_empty())
        })
        .count();

    for (label, value) in [
        ("Version   ", env!("CARGO_PKG_VERSION").to_string()),
        (
            "Platform  ",
            format!("{} / {}", std::env::consts::OS, std::env::consts::ARCH),
        ),
        (
            "Snapshot  ",
            format!(
                "{} processes · {listener_summary}",
                snapshot.processes_by_pid.len(),
            ),
        ),
        (
            "Tools     ",
            format!(
                "{} of {} found on PATH",
                tools_found,
                snapshot.runtimes.len()
            ),
        ),
    ] {
        lines.push(Line::from(vec![
            Span::styled(label, Style::new().fg(BLUE).bold()),
            Span::raw(value),
        ]));
    }
    Text::from(lines)
}

fn draw_footer(frame: &mut Frame, area: Rect, focus: Focus) {
    let brand = format!("DevDoctor {}", env!("CARGO_PKG_VERSION"));
    // On narrow terminals, give actual shortcuts priority over branding.
    let brand_width = if area.width >= 100 {
        brand.len() as u16
    } else {
        0
    };
    let [shortcuts, branding] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(brand_width)])
            .spacing(u16::from(brand_width > 0))
            .areas(area);

    let mut hints = vec![Span::styled("1/2/3 h/l Tab", Style::new().fg(BLUE))];
    if focus != Focus::Status {
        hints.extend([
            Span::styled(" │ ", Style::new().fg(MUTED)),
            Span::styled("Move: j/k", Style::new().fg(BLUE)),
            Span::styled(" │ ", Style::new().fg(MUTED)),
            Span::styled("Search: /", Style::new().fg(BLUE)),
        ]);
    }
    if focus == Focus::Processes {
        hints.extend([
            Span::styled(" │ ", Style::new().fg(MUTED)),
            Span::styled("Filter: [ ]", Style::new().fg(BLUE)),
        ]);
    }
    hints.extend([
        Span::styled(" │ ", Style::new().fg(MUTED)),
        Span::styled("Refresh: r", Style::new().fg(BLUE)),
        Span::styled(" │ ", Style::new().fg(MUTED)),
        Span::styled("Quit: q", Style::new().fg(BLUE)),
    ]);

    frame.render_widget(Line::from(hints), shortcuts);
    frame.render_widget(
        Line::styled(brand, Style::new().fg(PINK)).right_aligned(),
        branding,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{
        Terminal,
        backend::{Backend, TestBackend},
        widgets::ListState,
    };

    #[test]
    fn search_titles_fit_the_available_space_and_preserve_casing() {
        for query in [
            "Node",
            "工具Python",
            "a very long query that must be shortened",
        ] {
            for width in [0, 4, 12, 40] {
                let title = search_title(query, false, width);
                assert!(title.width() <= width);
            }
        }
        assert_eq!(search_title("Node", false, 40).to_string(), " /Node ");
        assert!(search_title("", false, 40).to_string().is_empty());
        assert_eq!(search_title("", true, 40).to_string(), " / ");
        assert!(
            search_title("long query", false, 8)
                .to_string()
                .contains('…')
        );
    }

    #[test]
    fn search_editor_keeps_its_cursor_visible_for_long_and_unicode_queries() {
        for query in [String::new(), "Node工具".to_string(), "x".repeat(120)] {
            for width in [1, 20, 80] {
                let mut terminal = Terminal::new(TestBackend::new(width, 2)).unwrap();
                terminal
                    .draw(|frame| {
                        draw_search_editor(
                            frame,
                            Rect::new(0, 0, width, 1),
                            Rect::new(0, 1, width, 1),
                            Focus::Processes,
                            &query,
                        );
                    })
                    .unwrap();
                let cursor = terminal.backend_mut().get_cursor_position().unwrap();
                assert!(cursor.x < width);
                assert_eq!(cursor.y, 0);
            }
        }
    }

    #[test]
    fn rounded_border_tracks_focus() {
        for (focused, color) in [(true, PINK), (false, BORDER)] {
            let mut terminal = Terminal::new(TestBackend::new(40, 8)).unwrap();
            terminal
                .draw(|frame| {
                    frame.render_widget(panel(Line::from("Processes"), focused), frame.area());
                })
                .unwrap();

            let corner = &terminal.backend().buffer()[(0, 0)];
            assert_eq!(corner.symbol(), "╭");
            assert_eq!(corner.fg, color);
        }
    }

    #[test]
    fn selected_row_is_subtler_in_an_unfocused_panel() {
        for (focused, background) in [(true, SELECTION), (false, INACTIVE_SELECTION)] {
            let mut terminal = Terminal::new(TestBackend::new(40, 8)).unwrap();
            let mut state = ListState::default().with_selected(Some(0));
            terminal
                .draw(|frame| {
                    let list = List::new(["node", "python"])
                        .block(panel(Line::from("Processes"), focused))
                        .highlight_style(selection_style(focused));
                    frame.render_stateful_widget(list, frame.area(), &mut state);
                })
                .unwrap();

            assert_eq!(terminal.backend().buffer()[(1, 1)].bg, background);
            assert_eq!(state.selected(), Some(0));
        }
    }

    #[test]
    fn counters_handle_empty_lists_and_pending_navigation() {
        assert_eq!(position(None, 0).to_string(), " 0 of 0 ");
        assert_eq!(position(Some(0), 0).to_string(), " 0 of 0 ");
        assert_eq!(position(Some(2), 4).to_string(), " 3 of 4 ");
        assert_eq!(position(Some(usize::MAX), 4).to_string(), " 4 of 4 ");
    }

    #[test]
    fn details_preserve_multiline_values() {
        let text = detail_sections([("TCP LISTENERS", "127.0.0.1:3000\n[::1]:8080".to_string())]);
        assert_eq!(text.lines.len(), 3);
        assert_eq!(text.lines[0].to_string(), "TCP LISTENERS");
        assert_eq!(text.lines[1].to_string(), "127.0.0.1:3000");
        assert_eq!(text.lines[2].to_string(), "[::1]:8080");
    }

    #[test]
    fn footer_hints_follow_focus_and_fit_tiny_terminals_without_panicking() {
        for focus in [Focus::Status, Focus::Processes, Focus::Runtimes] {
            for width in [1, 20, 80, 120] {
                let mut terminal = Terminal::new(TestBackend::new(width, 1)).unwrap();
                terminal
                    .draw(|frame| draw_footer(frame, frame.area(), focus))
                    .unwrap();

                if width >= 80 {
                    let buffer = terminal.backend().buffer();
                    let content: String = (0..width).map(|x| buffer[(x, 0)].symbol()).collect();
                    assert_eq!(content.contains("Filter: [ ]"), focus == Focus::Processes);
                    assert_eq!(content.contains("Move: j/k"), focus != Focus::Status);
                    assert_eq!(content.contains("Search: /"), focus != Focus::Status);
                    assert!(content.contains("1/2/3 h/l Tab"));
                    assert!(content.contains("Quit: q"));
                    assert_eq!(content.contains("DevDoctor"), width >= 100);
                }
            }
        }
    }

    #[test]
    fn status_uses_a_compact_logo_when_the_panel_is_narrow() {
        let snapshot = SystemSnapshot {
            runtimes: Vec::new(),
            processes_by_pid: Default::default(),
            sockets_by_pid: Default::default(),
            listener_error: None,
        };
        let full = status_details(&snapshot, 100).to_string();
        assert!(full.contains(LOGO));
        let compact = status_details(&snapshot, 20).to_string();
        assert!(compact.contains("[ DevDoctor ]"));
        assert!(!compact.contains(LOGO));
        assert!(compact.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn status_reports_listener_discovery_failures_instead_of_zero_listeners() {
        let snapshot = SystemSnapshot {
            runtimes: Vec::new(),
            processes_by_pid: Default::default(),
            sockets_by_pid: Default::default(),
            listener_error: Some("permission denied".to_string()),
        };
        let status = status_details(&snapshot, 100).to_string();
        assert!(status.contains("listener discovery failed: permission denied"));
        assert!(!status.contains("0 TCP listeners"));
    }
}
