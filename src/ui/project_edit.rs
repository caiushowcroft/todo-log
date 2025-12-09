use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(3),  // Name field
            Constraint::Length(5),  // Description field (multi-line)
            Constraint::Length(3),  // Jira field
            Constraint::Length(3),  // Status field
            Constraint::Length(3),  // Group field
            Constraint::Min(1),     // Spacer
            Constraint::Length(3),  // Help bar
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Edit Project")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Name field (field 0)
    let name_style = if app.project_edit_field == 0 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let name_field = Paragraph::new(app.project_edit_name.as_str())
        .style(name_style)
        .block(Block::default().borders(Borders::ALL).title("[1] Name *"));
    frame.render_widget(name_field, chunks[1]);

    // Description field (field 1)
    let desc_style = if app.project_edit_field == 1 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let desc_field = Paragraph::new(app.project_edit_description.as_str())
        .style(desc_style)
        .block(Block::default().borders(Borders::ALL).title("[2] Description"));
    frame.render_widget(desc_field, chunks[2]);

    // Jira field (field 2)
    let jira_style = if app.project_edit_field == 2 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let jira_field = Paragraph::new(app.project_edit_jira.as_str())
        .style(jira_style)
        .block(Block::default().borders(Borders::ALL).title("[3] Jira URL"));
    frame.render_widget(jira_field, chunks[3]);

    // Status field (field 3)
    let status_style = if app.project_edit_field == 3 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let status_field = Paragraph::new(app.project_edit_status.as_str())
        .style(status_style)
        .block(Block::default().borders(Borders::ALL).title("[4] Status"));
    frame.render_widget(status_field, chunks[4]);

    // Group field (field 4)
    let group_style = if app.project_edit_field == 4 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let group_field = Paragraph::new(app.project_edit_group.as_str())
        .style(group_style)
        .block(Block::default().borders(Borders::ALL).title("[5] Group"));
    frame.render_widget(group_field, chunks[5]);

    // Help bar
    let help_text = vec![
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(" Next field  "),
        Span::styled("1-5", Style::default().fg(Color::Yellow)),
        Span::raw(" Jump to field  "),
        Span::styled("Ctrl+S", Style::default().fg(Color::Yellow)),
        Span::raw(" Save  "),
        Span::styled("ESC", Style::default().fg(Color::Yellow)),
        Span::raw(" Cancel"),
    ];
    let help = Paragraph::new(Line::from(help_text))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[7]);

    // Set cursor position for active field
    let (field_area, input) = match app.project_edit_field {
        0 => (chunks[1], &app.project_edit_name),
        1 => (chunks[2], &app.project_edit_description),
        2 => (chunks[3], &app.project_edit_jira),
        3 => (chunks[4], &app.project_edit_status),
        4 => (chunks[5], &app.project_edit_group),
        _ => (chunks[1], &app.project_edit_name),
    };

    let inner = field_area.inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 });
    frame.set_cursor_position((
        inner.x + input.len() as u16,
        inner.y,
    ));
}
