extern crate dirs;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::time::Instant;
use std::{io, time::Duration};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans, Text};
use tui::widgets::{Clear, List, ListItem, Paragraph};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
    Frame, Terminal,
};

use file_manager::app::App;
use file_manager::InputMode;

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App, mut curr_file: String, curr_file_name: String) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Percentage(70),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());
    // show error if there is one
    if app.error != None {
        curr_file = app.error.clone().unwrap();
    }
    let mut sp = vec![];
    for line in curr_file.lines() {
        sp.push(Spans::from(line));
    }
    let p = Paragraph::new(sp).block(Block::default().title(curr_file_name).borders(Borders::ALL));
    f.render_widget(p, chunks[1]);
    // f.render_widget(p, chunks[2]);
    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = app
        .list
        .items
        .iter()
        .map(|i| {
            let mut lines = vec![Spans::from(i.file_name().unwrap().to_str().unwrap())];
            // mark folders
            if i.is_dir() {
                lines[0] = Spans::from(Span::styled(
                    i.file_name().unwrap().to_str().unwrap(),
                    Style::default()
                        .fg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                ));
            }
            ListItem::new(lines).style(Style::default().fg(Color::White).bg(Color::Black))
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(app.list.curr_dir.clone().to_string()),
        )
        .highlight_style(
            Style::default()
                .bg(Color::LightBlue)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::ITALIC),
        )
        .highlight_symbol(">> ");

    // We can now render the item list
    f.render_stateful_widget(items, chunks[0], &mut app.list.state);
    if app.show_popup {
        let block = Block::default().title("Popup").borders(Borders::ALL);
        let area = centered_rect(60, 20, f.size());
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(block, area);
        // show text box
        let area = centered_rect(58, 18, f.size());
        let mut text = Text::from(app.input.clone());
        text.patch_style(Style::default().fg(Color::LightBlue));
        let p = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
        f.render_widget(p, area);
    }
}

fn main() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let mut app = App::new();
    // let res = run_app(&mut terminal, app, tick_rate);

    let mut last_tick = Instant::now();
    // select the first item
    app.list.state.select(Some(0));
    let mut curr_file: String = String::from("");
    let mut curr_file_name: String = String::from("");
    loop {
        terminal.draw(|f| {
            ui(f, &mut app, curr_file.clone(), curr_file_name.clone());
        })?;
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Left => app.go_back(),
                        KeyCode::Down => app.list.next(),
                        KeyCode::Up => app.list.previous(),
                        KeyCode::Enter | KeyCode::Right => {
                            // first check if its a file
                            if app.list.items[app.list.state.selected().unwrap()].is_file() {
                                app.error = None;
                                // open file
                                let mut path = PathBuf::from(&app.list.curr_dir);
                                path.push(
                                    app.list.items[app.list.state.selected().unwrap()]
                                        .file_name()
                                        .unwrap(),
                                );
                                let mut file = match File::open(path.clone()) {
                                    Err(why) => {
                                        app.error = Option::from(format!(
                                            "couldn't open {}: {}",
                                            path.display(),
                                            why.to_string()
                                        ));
                                        return Ok(());
                                    }
                                    Ok(file) => file,
                                };
                                curr_file.clear();
                                match file.read_to_string(&mut curr_file) {
                                    Err(why) => {
                                        app.error = Option::from(format!(
                                            "couldn't open {}: {}",
                                            path.display(),
                                            why.to_string()
                                        ))
                                    }
                                    Ok(_) => {}
                                };
                                curr_file_name = app.list.items[app.list.state.selected().unwrap()]
                                    .file_name()
                                    .unwrap()
                                    .to_str()
                                    .unwrap()
                                    .to_string();
                                // println!("{}", contents);
                                // show content in the right pane
                            } else {
                                app.go_into();
                            }
                        }
                        KeyCode::Char('n') => app.new_folder(),
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => {
                            // callback
                            app.callback.as_mut().unwrap().as_ref()(app.input.clone());
                            app.input_mode = InputMode::Normal;
                            app.show_popup = false;
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                            app.show_popup = false;
                        }
                        _ => {}
                    },
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
