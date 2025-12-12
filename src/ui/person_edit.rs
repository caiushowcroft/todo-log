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
            Constraint::Length(3),  // Full name field
            Constraint::Length(3),  // Email field
            Constraint::Length(3),  // Tel field
            Constraint::Length(3),  // Company field
            Constraint::Min(1),     // Spacer
            Constraint::Length(3),  // Help bar
        ])
        .split(area);

    // Title - show different text for new vs edit
    let title_text = match app.screen {
        super::app::Screen::PersonEdit(None) => "New Person",
        _ => "Edit Person",
    };
    let title = Paragraph::new(title_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Name field (field 0)
    let name_style = if app.person_edit_field == 0 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let name_field = Paragraph::new(app.person_edit_name.as_str())
        .style(name_style)
        .block(Block::default().borders(Borders::ALL).title("[1] Name *"));
    frame.render_widget(name_field, chunks[1]);

    // Full name field (field 1)
    let full_name_style = if app.person_edit_field == 1 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let full_name_field = Paragraph::new(app.person_edit_full_name.as_str())
        .style(full_name_style)
        .block(Block::default().borders(Borders::ALL).title("[2] Full Name"));
    frame.render_widget(full_name_field, chunks[2]);

    // Email field (field 2)
    let email_style = if app.person_edit_field == 2 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let email_field = Paragraph::new(app.person_edit_email.as_str())
        .style(email_style)
        .block(Block::default().borders(Borders::ALL).title("[3] Email"));
    frame.render_widget(email_field, chunks[3]);

    // Tel field (field 3)
    let tel_style = if app.person_edit_field == 3 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let tel_field = Paragraph::new(app.person_edit_tel.as_str())
        .style(tel_style)
        .block(Block::default().borders(Borders::ALL).title("[4] Tel"));
    frame.render_widget(tel_field, chunks[4]);

    // Company field (field 4)
    let company_style = if app.person_edit_field == 4 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let company_field = Paragraph::new(app.person_edit_company.as_str())
        .style(company_style)
        .block(Block::default().borders(Borders::ALL).title("[5] Company"));
    frame.render_widget(company_field, chunks[5]);

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
    let (field_area, input) = match app.person_edit_field {
        0 => (chunks[1], &app.person_edit_name),
        1 => (chunks[2], &app.person_edit_full_name),
        2 => (chunks[3], &app.person_edit_email),
        3 => (chunks[4], &app.person_edit_tel),
        4 => (chunks[5], &app.person_edit_company),
        _ => (chunks[1], &app.person_edit_name),
    };

    let inner = field_area.inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 });
    frame.set_cursor_position((
        inner.x + input.len() as u16,
        inner.y,
    ));
}
