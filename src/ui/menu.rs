use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Menu options
            Constraint::Length(3),  // Status
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Todo-Log")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Menu options - numbered list with selection
    let menu_options = [
        "Create a new log entry",
        "List current todos",
        "Show logs by project/person",
        "View projects",
        "View people",
    ];

    let mut menu_items = Vec::new();
    for (i, option) in menu_options.iter().enumerate() {
        let is_selected = i == app.menu_selected;
        let style = if is_selected {
            Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        menu_items.push(Line::from(vec![
            Span::styled(format!("{}. ", i + 1), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(*option, style),
        ]));
        menu_items.push(Line::from(""));
    }

    let menu = Paragraph::new(menu_items)
        .block(Block::default().borders(Borders::ALL).title("Menu (↑↓ to navigate, Enter to select, ESC to quit)"))
        .alignment(Alignment::Left);
    frame.render_widget(menu, chunks[1]);

    // Status bar
    let status_text = app.status_message.as_deref().unwrap_or("Ready");
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(status, chunks[2]);
}
