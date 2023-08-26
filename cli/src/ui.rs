use tui::style::{Color, Style};
use tui::widgets::{Block, Borders};

pub(crate) const NAV_SECTIONS: &[&str] = &[
    "General",
    "Archive",
    "Stats",
    "Hardware",
    "Admin"
];

pub(crate) const NAV_WIDTH: u16 = 15;

pub(crate) fn generate_nav_box<'a>(selected: bool) -> Block<'a> {
    Block::default()
        .title("Navigation")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .border_style(Style::default().fg(if selected { Color::Yellow } else { Color::White }))
}

pub(crate) const FOOTER_HEIGHT: u16 = 3;
pub(crate) const TOP_HEIGHT: u16 = 3;

pub(crate) mod top_informations {
    use std::io::Stdout;
    use tui::backend::CrosstermBackend;
    use tui::Frame;
    use tui::layout::Rect;
    use tui::style::{Color, Style};
    use tui::text::Text;
    use tui::widgets::{Block, Borders, Paragraph};
    use crate::ui::{NAV_WIDTH, TOP_HEIGHT};

    fn gen_block<'a>() -> Block<'a> {
        Block::default()
            .title("Informations")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .border_style(Style::default().fg(Color::White))
    }

    pub(crate) fn draw(f: &mut Frame<CrosstermBackend<Stdout>>, size: Rect, section: usize) {
        let info_block = gen_block();

        f.render_widget(
            info_block,
            Rect {
                x: NAV_WIDTH,
                y: 0,
                width: size.width - NAV_WIDTH - 2,
                height: TOP_HEIGHT
            }
        );

        let inner_rect = Rect::new(size.x + 1 + NAV_WIDTH + 2, size.y + 1, size.width - NAV_WIDTH - 4, TOP_HEIGHT);
        // draw section name
        let text = match section {
            0 => "General - Contains general informations about kady",
            _ => "Unknown"
        };

        let paragraph = Paragraph::new(Text::raw(text))
            .block(Block::default().borders(Borders::NONE));

        f.render_widget(paragraph, inner_rect);
    }
}

pub(crate) mod general {
    use std::io::Stdout;
    use std::sync::{Arc, Mutex};
    use tui::backend::CrosstermBackend;
    use tui::Frame;
    use tui::layout::Rect;
    use tui::style::{Color, Style};
    use tui::widgets::{Block, Borders, List, ListItem};
    use config::Config;

    fn gen_block<'a>(selected: bool) -> Block<'a> {
        Block::default()
            .title("General")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if selected { Color::Yellow } else { Color::White }))
    }

    pub(crate) fn draw(f: &mut Frame<CrosstermBackend<Stdout>>, size: Rect, selected: bool, config: Arc<Mutex<Config>>) {
        let block = gen_block(selected);

        f.render_widget(block, size);

        let inner_rect = Rect::new(size.x + 2, size.y + 2, size.width - 4, size.height - 4);

        let config = config.lock().unwrap();
        let mut list_items: Vec<ListItem> = Vec::new();
        list_items.push(ListItem::new(format!("Version:      {}", config.version)));
        list_items.push(ListItem::new(format!("Build:        {}", config.build)));
        list_items.push(ListItem::new(format!("\nTraductions:  {}", config.langs)));
        list_items.push(ListItem::new(format!("Archive path: {}", config.security.archive_path)));

        let list = List::new(list_items)
            .block(Block::default().borders(Borders::NONE))
            .highlight_style(Style::default().fg(Color::Yellow));

        f.render_widget(list, inner_rect);
    }
}


pub(crate) mod archive {
    use std::io::Stdout;
    use std::sync::{Arc, Mutex};
    use tui::backend::CrosstermBackend;
    use tui::Frame;
    use tui::layout::Rect;
    use tui::style::{Color, Style};
    use tui::text::Text;
    use tui::widgets::{Block, Borders, Paragraph};
    use tui_textarea::TextArea;

    fn gen_block<'a>(selected: bool, sudo: bool) -> Block<'a> {
        Block::default()
            .title("Archive [restricted]")
            .borders(Borders::ALL)
            .border_style(
                Style::default().fg(
                    if !sudo { if selected { Color::Red } else { Color::LightRed } }
                    else if selected { Color::Yellow } else { Color::White }
                )
            )
    }

    fn access_denied(f: &mut Frame<CrosstermBackend<Stdout>>, inner: Rect){
        let text = "Access denied";
        let paragraph = Paragraph::new(Text::raw(text))
            .alignment(tui::layout::Alignment::Center)
            .style(Style::default().fg(Color::Red));

        f.render_widget(paragraph, inner);
    }

    pub(crate) fn draw(
        f: &mut Frame<CrosstermBackend<Stdout>>,
        size: Rect,
        selected: bool,
        sudo: bool,
        archive_text_area: Arc<Mutex<TextArea>>
    ) {
        let block = gen_block(selected, sudo);

        f.render_widget(block, size);
        let inner_rect = Rect::new(size.x + 2, size.y + 2, size.width - 4, size.height - 4);

        if !sudo {
            access_denied(f, inner_rect);
            return;
        }

        let archive_text_area = archive_text_area.lock().unwrap();

        f.render_widget(archive_text_area.clone().widget(), inner_rect);
    }
}


pub(crate) mod footer {
    use std::io::Stdout;
    use tui::backend::CrosstermBackend;
    use tui::Frame;
    use tui::layout::Rect;
    use tui::text::Text;
    use tui::widgets::{Block, Borders, Paragraph};
    use crate::ui::{FOOTER_HEIGHT, NAV_WIDTH};

    fn gen_block<'a>() -> Block<'a> {
        Block::default().borders(Borders::ALL)
    }

    pub(crate) fn draw(f: &mut Frame<CrosstermBackend<Stdout>>, size: Rect) {
        let block = gen_block();

        f.render_widget(block, size);

        let inner_rect = Rect::new(NAV_WIDTH + 2, size.y + 1, size.width - NAV_WIDTH - 4, FOOTER_HEIGHT);
        // draw section name
        let text = "'q' to quit, navigate with arrows, 'enter' to select";

        let paragraph = Paragraph::new(Text::raw(text))
            .block(Block::default().borders(Borders::NONE));

        f.render_widget(paragraph, inner_rect);
    }
}