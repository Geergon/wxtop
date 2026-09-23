use ratatui::{
    layout::{
        Alignment, Constraint,
        Direction::{self, Vertical},
        Layout, Rect,
    },
    macros::constraints,
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Clear, List, ListItem, Paragraph, Widget},
    Frame,
};

use crate::app::{CurrentScreen, Model};

pub fn view(model: &mut Model, frame: &mut Frame) {
    match model.current_screen {
        CurrentScreen::Prompt => initial_prompt(model, frame),
        CurrentScreen::Main => main_screen(model, frame),
    }
}

fn main_screen(model: &mut Model, frame: &mut Frame) {}

fn initial_prompt(model: &mut Model, frame: &mut Frame) {
    let prompt_area = centered_rect(80, 70, frame.area());
    let chunks = Layout::default()
        .direction(Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(prompt_area);

    let instruction = Line::from(vec![
        " Select ".yellow(),
        "[↑/↓]".blue().bold(),
        " Confirm ".yellow(),
        "[Enter]".blue().bold(),
        " Quit ".yellow(),
        "[Q]".blue().bold(),
    ]);

    let search_block = Block::bordered()
        .border_set(border::ROUNDED)
        .title(Line::from("Search").left_aligned())
        .title_bottom(instruction)
        .title_alignment(Alignment::Right);

    let search_text = if model.init_prompt_state.input.user_input.is_empty() {
        Paragraph::new("Type to search...").block(search_block)
    } else {
        Paragraph::new(model.init_prompt_state.input.user_input.to_owned()).block(search_block)
    };
    frame.render_widget(search_text, chunks[1]);

    if !model.init_prompt_state.input.user_input.is_empty() {
        let cursor_x = chunks[1].x + 1 + model.init_prompt_state.input.character_index as u16;
        let cursor_y = chunks[1].y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    }

    let result_block = Block::bordered().border_set(border::ROUNDED);

    let items: Vec<ListItem> = model
        .init_prompt_state
        .list_entry
        .iter()
        .map(|entry| ListItem::new(entry.to_owned()))
        .collect();

    let list = List::new(items)
        .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black))
        .block(result_block);

    frame.render_stateful_widget(list, chunks[0], &mut model.init_prompt_state.list_state);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
