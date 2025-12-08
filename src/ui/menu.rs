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

    // Menu options
    let menu_items = vec![
        Line::from(vec![
            Span::styled("[c]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Create a new log entry"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("[d]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" List current todos"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("[s]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Show logs by project/person"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("[p]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" View projects"),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("[q]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" Quit"),
        ]),
    ];

    let menu = Paragraph::new(menu_items)
        .block(Block::default().borders(Borders::ALL).title("Menu"))
        .alignment(Alignment::Left);
    frame.render_widget(menu, chunks[1]);

    // Status bar
    let status_text = app.status_message.as_deref().unwrap_or("Ready");
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(status, chunks[2]);
}
