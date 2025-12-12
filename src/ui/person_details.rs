use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use super::app::App;

pub fn render(frame: &mut Frame, app: &mut App, area: Rect, person_idx: usize) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(12), // Person info
            Constraint::Min(10),    // Log list
            Constraint::Length(3),  // Help bar
        ])
        .split(area);

    // Render person info
    if let Some(person) = app.people.get(person_idx) {
        render_person_info(frame, person, chunks[0]);
    }

    // Render log list
    render_log_list(frame, app, chunks[1]);

    // Help bar
    let help_text = vec![
        Span::styled("↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(" Navigate  "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" View log  "),
        Span::styled("e", Style::default().fg(Color::Yellow)),
        Span::raw(" Edit person  "),
        Span::styled("ESC", Style::default().fg(Color::Yellow)),
        Span::raw(" Back"),
    ];
    let help = Paragraph::new(Line::from(help_text))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[2]);
}

fn render_person_info(frame: &mut Frame, person: &crate::models::Person, area: Rect) {
    let info_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Name
            Constraint::Length(3), // Full name
            Constraint::Length(3), // Email
            Constraint::Length(3), // Tel and Company (side by side)
        ])
        .split(area);

    // Name
    let name = Paragraph::new(&person.name as &str)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title("Name"));
    frame.render_widget(name, info_chunks[0]);

    // Full name
    let full_name_text = person.full_name.as_ref()
        .map(|f| f.as_str())
        .unwrap_or("(no full name)");
    let full_name = Paragraph::new(full_name_text)
        .block(Block::default().borders(Borders::ALL).title("Full Name"));
    frame.render_widget(full_name, info_chunks[1]);

    // Email
    let email_text = person.email.as_ref()
        .map(|e| e.as_str())
        .unwrap_or("(no email)");
    let email = Paragraph::new(email_text)
        .style(Style::default().fg(Color::Blue))
        .block(Block::default().borders(Borders::ALL).title("Email"));
    frame.render_widget(email, info_chunks[2]);

    // Tel and Company side by side
    let bottom_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Tel
            Constraint::Percentage(50), // Company
        ])
        .split(info_chunks[3]);

    let tel_text = person.tel.as_ref()
        .map(|t| t.as_str())
        .unwrap_or("(no tel)");
    let tel = Paragraph::new(tel_text)
        .block(Block::default().borders(Borders::ALL).title("Tel"));
    frame.render_widget(tel, bottom_row[0]);

    let company_text = person.company.as_ref()
        .map(|c| c.as_str())
        .unwrap_or("(no company)");
    let company = Paragraph::new(company_text)
        .block(Block::default().borders(Borders::ALL).title("Company"));
    frame.render_widget(company, bottom_row[1]);
}

fn render_log_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .person_details_logs
        .iter()
        .enumerate()
        .map(|(i, log)| {
            let timestamp = log.timestamp.format("%Y-%m-%d %H:%M").to_string();
            let first_line = log.first_line();
            let preview: String = first_line.chars().take(50).collect();

            let tags = format!(
                " #{}",
                if log.projects.is_empty() {
                    "-".to_string()
                } else {
                    log.projects.join(" #")
                }
            );

            let style = if i == app.person_details_log_selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::styled(timestamp, Style::default().fg(Color::Cyan)),
                Span::raw(" | "),
                Span::styled(preview, style),
                Span::styled(tags, Style::default().fg(Color::Green)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.person_details_log_selected));

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!("Log Entries ({} total)", app.person_details_logs.len())))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(list, area, &mut state);
}
