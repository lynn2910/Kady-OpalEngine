mod ui;

use std::{io, thread};
use std::io::Stdout;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tui::{backend::CrosstermBackend, Frame, Terminal};
use crossterm::{event::{EnableMouseCapture, DisableMouseCapture, KeyCode}, event, execute, terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen }};
use crossterm::event::Event;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::text::Text;
use tui::widgets::{Block, Borders, List, ListItem, Paragraph};
use tui_textarea::{TextArea, Input, Key, CursorMove};
use archive::Archive;
use crate::ui::{FOOTER_HEIGHT, NAV_SECTIONS, NAV_WIDTH, TOP_HEIGHT};

const SUDO_TEXTAREA_WIDTH: u16 = 30;
const SUDO_TEXTAREA_HEIGHT: u16 = 3;

const SUDO_SECTIONS: &[usize] = &[1];

fn main() -> Result<(), io::Error> {
    // Create backend and terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let config = config::load_from("./config.toml".into()).expect("Failed to load config at './archive'");

    let archive = Archive::open(
        PathBuf::from_str(config.security.archive_path.as_str())
            .expect("Cannot create PathBuf for archive")
    ).expect("Failed to load archive");

    let config = Arc::new(Mutex::new(config));
    let archive = Arc::new(Mutex::new(archive));

    let sudo_textarea = {
        let mut sudo_textarea = TextArea::default();
        sudo_textarea.set_cursor_style(Style::default());
        sudo_textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Sudo access")
                .border_style(Style::default().fg(Color::Yellow))
                .style(Style::default().fg(Color::Yellow))
        );
        sudo_textarea.set_alignment(tui::layout::Alignment::Left);
        Arc::new(Mutex::new(sudo_textarea))
    };

    let archive_textarea = {
        let mut archive_textarea = TextArea::default();
        archive_textarea.set_cursor_style(Style::default().fg(Color::Yellow).bg(Color::Yellow));
        archive_textarea.set_alignment(tui::layout::Alignment::Left);
        archive_textarea.set_block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default().fg(Color::White))
        );
        Arc::new(Mutex::new(archive_textarea))
    };

    let sudo_access_granted = Arc::new(Mutex::new(false));
    let sudo_request_opened = Arc::new(Mutex::new(false));

    // Navigation variables
    let selected_section = Arc::new(Mutex::new(0));
    let actual_section = Arc::new(Mutex::new(0));
    let run_cli = Arc::new(Mutex::new(true));
    let inside_section = Arc::new(Mutex::new(false));

    let selected_section_clone = selected_section.clone();
    let actual_section_clone = actual_section.clone();
    let inside_section_clone = inside_section.clone();
    let run_cli_clone = run_cli.clone();

    let sudo_textarea_clone = sudo_textarea.clone();
    let sudo_access_granted_clone = sudo_access_granted.clone();
    let sudo_request_opened_clone = sudo_request_opened.clone();

    let archive_textarea_clone = archive_textarea.clone();
    let archive_clone = archive.clone();

    // The input handler thread
    thread::spawn(move || {
        loop {
            if !*run_cli_clone.lock().unwrap() {
                break;
            }

            #[allow(clippy::collapsible_match)]
            if let Ok(event) = event::read() {
                if let Event::Key(event) = event {
                    {
                        let mut input = Input::default();
                        if let KeyCode::Char(c) = event.code { input.key = Key::Char(c); }
                        if let KeyCode::Delete = event.code { input.key = Key::Delete; }
                        if let KeyCode::Backspace = event.code { input.key = Key::Backspace; }
                        if let KeyCode::Left = event.code { input.key = Key::Left; }
                        if let KeyCode::Right = event.code { input.key = Key::Right; }
                        if let KeyCode::Up = event.code { input.key = Key::Up; }
                        if let KeyCode::Down = event.code { input.key = Key::Down; }
                        if let KeyCode::Tab = event.code { input.key = Key::Tab; }
                        if let KeyCode::Enter = event.code { input.key = Key::Enter; }

                        let sudo_request_opened = sudo_request_opened_clone.lock().unwrap();
                        let actual_section = actual_section_clone.lock().unwrap();
                        if *sudo_request_opened {
                            let mut text_area = sudo_textarea_clone.lock().unwrap();
                            text_area.input(input.clone());
                        } else if *actual_section == 1 {
                            let mut text_area = archive_textarea_clone.lock().unwrap();
                            text_area.input(input.clone());
                        }
                    }

                    match event.code {
                        KeyCode::Char('q') => {
                            *run_cli_clone.lock().unwrap() = false;
                            break;
                        },
                        KeyCode::Up | KeyCode::PageUp => {
                            let inside_section = inside_section_clone.lock().unwrap();
                            if !*inside_section {
                                let mut selected_section = selected_section_clone.lock().unwrap();
                                let mut actual_section = actual_section_clone.lock().unwrap();
                                *selected_section = (*selected_section + NAV_SECTIONS.len() - 1) % NAV_SECTIONS.len();
                                *actual_section = *selected_section;
                            }
                        },
                        KeyCode::Down | KeyCode::PageDown => {
                            let inside_section = inside_section_clone.lock().unwrap();
                            if !*inside_section {
                                let mut selected_section = selected_section_clone.lock().unwrap();
                                let mut actual_section = actual_section_clone.lock().unwrap();
                                *selected_section = (*selected_section + 1) % NAV_SECTIONS.len();
                                *actual_section = *selected_section;
                            }
                        },
                        KeyCode::Enter => {
                            // navigate to the selected section
                            {
                                let mut inside_section = inside_section_clone.lock().unwrap();
                                if !*inside_section {
                                    let mut actual_section = actual_section_clone.lock().unwrap();
                                    *actual_section = *selected_section_clone.lock().unwrap();
                                    *inside_section = true;

                                    let sudo_access_granted = sudo_access_granted_clone.lock().unwrap();

                                    if !*sudo_access_granted && *actual_section == SUDO_SECTIONS[0] {
                                        let mut sudo_request_opened = sudo_request_opened_clone.lock().unwrap();
                                        *sudo_request_opened = true;
                                        continue;
                                    }
                                }
                            }
                            // sudo check
                            {
                                let actual_section = actual_section_clone.lock().unwrap();
                                let mut sudo_request_opened = sudo_request_opened_clone.lock().unwrap();
                                if *sudo_request_opened && SUDO_SECTIONS.contains(&*actual_section) {
                                    let mut sudo_textarea = sudo_textarea_clone.lock().unwrap();
                                    let entered_password = sudo_textarea.lines().join("");
                                    if check_sudo_access(entered_password.as_str()) {
                                        let mut sudo_access_granted = sudo_access_granted_clone.lock().unwrap();
                                        *sudo_access_granted = true;
                                        *sudo_request_opened = false;

                                        if *actual_section == 1 {
                                            let archive = archive_clone.lock().unwrap();
                                            let content = serde_json::to_string_pretty(&archive.copy_body()).unwrap();
                                            drop(archive);

                                            let mut archive_textarea = archive_textarea_clone.lock().unwrap();
                                            archive_textarea.move_cursor(CursorMove::Head);
                                            for line in content.lines() {
                                                archive_textarea.insert_str(line);
                                                archive_textarea.insert_newline();
                                            }
                                        }

                                    } else {
                                        sudo_textarea.move_cursor(CursorMove::Head);
                                        sudo_textarea.delete_line_by_end();
                                        sudo_textarea.set_block(
                                            Block::default()
                                                .borders(Borders::ALL)
                                                .title("Sudo access")
                                                .border_style(Style::default().fg(Color::Red))
                                                .style(Style::default().fg(Color::Red))
                                        );
                                    }
                                }
                            }
                        },
                        KeyCode::Esc => {
                            let mut inside_section = inside_section_clone.lock().unwrap();
                            if *inside_section { *inside_section = false; }

                            let mut sudo_request_opened = sudo_request_opened_clone.lock().unwrap();
                            if *sudo_request_opened {
                                *sudo_request_opened = false;
                                let mut sudo_textarea = sudo_textarea_clone.lock().unwrap();
                                sudo_textarea.move_cursor(CursorMove::Head);
                                sudo_textarea.delete_line_by_end();
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
    });

    // draw the UI
    let selected_section_clone = selected_section.clone();
    let actual_section_clone = actual_section.clone();
    let inside_section_clone = inside_section.clone();
    let run_cli_clone = run_cli.clone();

    let config_clone = config.clone();

    let sudo_textarea_clone = sudo_textarea.clone();
    let sudo_request_opened_clone = sudo_request_opened.clone();
    let sudo_access_granted_clone = sudo_access_granted.clone();

    let archive_textarea_clone = archive_textarea.clone();

    loop {
        if !*run_cli_clone.lock().unwrap() {
            break;
        }

        terminal.draw(|f| {
            let size = f.size();
            if size.height < 16 || size.width < 60 {
                terminal_too_small(f);
                return;
            }


            // draw navigation
            {
                let inside_section = inside_section_clone.lock().unwrap();
                let navigation_box = ui::generate_nav_box(!*inside_section);
                drop(inside_section);
                f.render_widget(navigation_box.clone(), Rect {
                    x: 0,
                    y: 0,
                    width: NAV_WIDTH,
                    height: f.size().height - 1
                });

                let selected_section = *selected_section_clone.lock().unwrap();

                let inner_rect = Rect::new(1, 1, NAV_WIDTH - 2, size.height - 2);

                let list_items: Vec<ListItem> = NAV_SECTIONS.iter().enumerate().map(|(i, section)| {
                    let mut item = ListItem::new(*section);
                    if i == selected_section { item = item.style(Style::default().fg(Color::Yellow)); }
                    item
                }).collect();

                let list = List::new(list_items)
                    .highlight_style(Style::default().bg(Color::White))
                    .highlight_symbol(">> ");

                f.render_widget(list, inner_rect);
            }
            // draw the panel
            {
                let actual_section = *actual_section_clone.lock().unwrap();
                ui::top_informations::draw(f, size, actual_section);
                let panel_size = Rect {
                    x: NAV_WIDTH,
                    y: TOP_HEIGHT,
                    width: size.width - NAV_WIDTH - 2,
                    height: size.height - FOOTER_HEIGHT - TOP_HEIGHT - 1
                };
                let inside_section = *inside_section_clone.lock().unwrap();
                match actual_section {
                    0 => ui::general::draw(f, panel_size, inside_section, config_clone.clone()),
                    1 => {
                        let sudo_access_granted = *sudo_access_granted_clone.lock().unwrap();
                        ui::archive::draw(
                            f,
                            panel_size,
                            inside_section,
                            sudo_access_granted,
                            archive_textarea_clone.clone()
                        );
                    },
                    _ => {}
                }
            }
            // draw the footer
            {
                let footer_rect = Rect {
                    x: NAV_WIDTH,
                    y: size.height - FOOTER_HEIGHT - 1,
                    width: size.width - NAV_WIDTH - 2,
                    height: FOOTER_HEIGHT
                };
                ui::footer::draw(f, footer_rect);
            }

            // draw sudo access if needed
            {
                let sudo_request_opened = sudo_request_opened_clone.lock().unwrap();
                if *sudo_request_opened {
                    // draw the textarea
                    let sudo_textarea = sudo_textarea_clone.lock().unwrap().clone();
                    let widget = sudo_textarea.widget();

                    f.render_widget(widget, Rect {
                        x: (size.width - SUDO_TEXTAREA_WIDTH) / 2,
                        y: (size.height - SUDO_TEXTAREA_HEIGHT) / 2,
                        width: SUDO_TEXTAREA_WIDTH,
                        height: SUDO_TEXTAREA_HEIGHT
                    });
                }
            }
        })?;
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

const SUDO_PWD: &str = "Rw0nhs87";

fn check_sudo_access(pwd: &str) -> bool {
    pwd == SUDO_PWD
}

fn terminal_too_small(f: &mut Frame<CrosstermBackend<Stdout>>) {
    let size = f.size();
    let text = "Terminal too small\nPlease resize the terminal to at least 60x16";
    let text_len = text.len() as u16;

    let block = Block::default()
        .title("Error")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Red));

    f.render_widget(block, size);

    f.render_widget(
        Paragraph::new(Text::raw(text))
            .style(Style::default().fg(Color::Red))
            .alignment(tui::layout::Alignment::Center),
        Rect::new(
            (size.width - text_len) / 2,
            size.height / 2,
            text_len,
            2
        )
    );
}