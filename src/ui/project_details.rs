use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use super::app::App;

pub fn render(frame: &mut Frame, app: &mut App, area: Rect, project_idx: usize) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(10), // Project info
            Constraint::Min(10),    // Log list
            Constraint::Length(3),  // Help bar
        ])
        .split(area);

    // Render project info
    if let Some(project) = app.projects.get(project_idx) {
        render_project_info(frame, app, project, chunks[0]);
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
        Span::raw(" Edit project  "),
        Span::styled("ESC", Style::default().fg(Color::Yellow)),
        Span::raw(" Back"),
    ];
    let help = Paragraph::new(Line::from(help_text))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[2]);
}

fn render_project_info(frame: &mut Frame, app: &App, project: &crate::models::Project, area: Rect) {
    let info_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Name/status and Group (side by side)
            Constraint::Length(3), // Jira
            Constraint::Min(3),    // Description
        ])
        .split(area);

    // Top row: Name/Status and Group side by side
    let top_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70), // Project name and status
            Constraint::Percentage(30), // Group
        ])
        .split(info_chunks[0]);

    // Name and status
    let status_color = app.config.get_state_color(&project.status);
    let name_line = Line::from(vec![
        Span::styled(&project.name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(
            format!("[{}]", project.status),
            Style::default().fg(status_color),
        ),
    ]);
    let name = Paragraph::new(name_line)
        .block(Block::default().borders(Borders::ALL).title("Project"));
    frame.render_widget(name, top_row[0]);

    // Group
    let group_text = if project.group.is_empty() {
        "(no group)".to_string()
    } else {
        project.group.clone()
    };
    let group = Paragraph::new(group_text)
        .block(Block::default().borders(Borders::ALL).title("Group"));
    frame.render_widget(group, top_row[1]);

    // Jira
    let jira_text = project.jira.as_ref()
        .map(|j| j.as_str())
        .unwrap_or("(no jira link)");
    let jira = Paragraph::new(jira_text)
        .style(Style::default().fg(Color::Blue))
        .block(Block::default().borders(Borders::ALL).title("Jira"));
    frame.render_widget(jira, info_chunks[1]);

    // Description
    let desc_text = project.description.as_ref()
        .map(|d| d.as_str())
        .unwrap_or("(no description)");
    let description = Paragraph::new(desc_text)
        .block(Block::default().borders(Borders::ALL).title("Description"))
        .wrap(ratatui::widgets::Wrap { trim: false });
    frame.render_widget(description, info_chunks[2]);
}

fn render_log_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .project_details_logs
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

            let style = if i == app.project_details_log_selected {
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
    state.select(Some(app.project_details_log_selected));

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!("Log Entries ({} total)", app.project_details_logs.len())))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(list, area, &mut state);
}
