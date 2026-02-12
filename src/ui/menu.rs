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

    // Status bar with stats
    let status_line = if let Some(ref msg) = app.status_message {
        Line::from(vec![Span::styled(msg.as_str(), Style::default().fg(Color::Yellow))])
    } else {
        // Show stats
        let log_count = app.storage.load_all_logs().map(|l| l.len()).unwrap_or(0);
        let todos = app.storage.load_all_todos().unwrap_or_default();
        let open_todos = todos.iter().filter(|t| !t.completed).count();
        let pinned_count = app.pins.len();

        Line::from(vec![
            Span::styled("📝 ", Style::default()),
            Span::styled(format!("{}", log_count), Style::default().fg(Color::Cyan)),
            Span::styled(" logs  ", Style::default().fg(Color::Gray)),
            Span::styled("☐ ", Style::default()),
            Span::styled(format!("{}", open_todos), Style::default().fg(Color::Yellow)),
            Span::styled(" open todos  ", Style::default().fg(Color::Gray)),
            Span::styled("📌 ", Style::default()),
            Span::styled(format!("{}", pinned_count), Style::default().fg(Color::Magenta)),
            Span::styled(" pinned", Style::default().fg(Color::Gray)),
        ])
    };

    let status = Paragraph::new(status_line)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(status, chunks[2]);
}
