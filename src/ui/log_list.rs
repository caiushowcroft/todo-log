use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use super::app::{App, LogFilterPanel};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(5),  // Filters summary
            Constraint::Min(10),    // Log list
            Constraint::Length(3),  // Help bar
        ])
        .split(area);

    // Title
    let title = Paragraph::new(format!("Logs ({} entries)", app.filtered_logs.len()))
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Filters summary
    render_filters_summary(frame, app, chunks[1]);

    // Log list
    render_log_list(frame, app, chunks[2]);

    // Help bar
    let help_text = if app.log_filter_panel == LogFilterPanel::None {
        vec![
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::raw(" Navigate  "),
            Span::styled("l", Style::default().fg(Color::Yellow)),
            Span::raw(" View  "),
            Span::styled("s", Style::default().fg(Color::Yellow)),
            Span::raw(" Start date  "),
            Span::styled("e", Style::default().fg(Color::Yellow)),
            Span::raw(" End date  "),
            Span::styled("p", Style::default().fg(Color::Yellow)),
            Span::raw(" Projects  "),
            Span::styled("h", Style::default().fg(Color::Yellow)),
            Span::raw(" People  "),
            Span::styled("ESC", Style::default().fg(Color::Yellow)),
            Span::raw(" Back"),
        ]
    } else {
        vec![
            Span::styled("ESC", Style::default().fg(Color::Yellow)),
            Span::raw(" Close filter"),
        ]
    };
    let help = Paragraph::new(Line::from(help_text))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[3]);

    // Render filter panel popup if active
    match app.log_filter_panel {
        LogFilterPanel::None => {}
        LogFilterPanel::StartDate | LogFilterPanel::EndDate => {
            render_date_filter_popup(frame, app, area);
        }
        LogFilterPanel::Projects => {
            render_project_filter_popup(frame, app, area);
        }
        LogFilterPanel::People => {
            render_people_filter_popup(frame, app, area);
        }
    }
}

fn render_filters_summary(frame: &mut Frame, app: &App, area: Rect) {
    let filter_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),  // Dates
            Constraint::Percentage(30),  // Projects
            Constraint::Percentage(30),  // People
        ])
        .split(area);

    // Date filter summary
    let start_str = app.log_filter.start_date
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "...".to_string());
    let end_str = app.log_filter.end_date
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "...".to_string());
    let date_text = format!("[s] {} to [e] {}", start_str, end_str);
    let date_style = match app.log_filter_panel {
        LogFilterPanel::StartDate | LogFilterPanel::EndDate => Style::default().fg(Color::Yellow),
        _ => Style::default(),
    };
    let dates = Paragraph::new(date_text)
        .style(date_style)
        .block(Block::default().borders(Borders::ALL).title("Dates"));
    frame.render_widget(dates, filter_chunks[0]);

    // Projects filter summary
    let projects_text = if app.log_filter.projects.is_empty() {
        "[p] All".to_string()
    } else {
        format!("[p] {}", app.log_filter.projects.join(", "))
    };
    let projects_style = if app.log_filter_panel == LogFilterPanel::Projects {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let projects = Paragraph::new(projects_text)
        .style(projects_style)
        .block(Block::default().borders(Borders::ALL).title("Projects"));
    frame.render_widget(projects, filter_chunks[1]);

    // People filter summary
    let people_text = if app.log_filter.people.is_empty() {
        "[h] All".to_string()
    } else {
        format!("[h] {}", app.log_filter.people.join(", "))
    };
    let people_style = if app.log_filter_panel == LogFilterPanel::People {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let people = Paragraph::new(people_text)
        .style(people_style)
        .block(Block::default().borders(Borders::ALL).title("People"));
    frame.render_widget(people, filter_chunks[2]);
}

fn render_date_filter_popup(frame: &mut Frame, app: &App, area: Rect) {
    let popup_width = 40u16;
    let popup_height = 8u16;
    let popup_x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Start date
            Constraint::Length(3),  // End date
        ])
        .split(popup_area);

    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title("Date Filter (YYYY-MM-DD)")
        .border_style(Style::default().fg(Color::Yellow));
    frame.render_widget(outer_block, popup_area);

    // Start date field
    let start_style = if app.log_filter_panel == LogFilterPanel::StartDate {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let start_field = Paragraph::new(app.start_date_input.as_str())
        .style(start_style)
        .block(Block::default().borders(Borders::ALL).title("[s] Start"));
    frame.render_widget(start_field, chunks[0]);

    // End date field
    let end_style = if app.log_filter_panel == LogFilterPanel::EndDate {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let end_field = Paragraph::new(app.end_date_input.as_str())
        .style(end_style)
        .block(Block::default().borders(Borders::ALL).title("[e] End"));
    frame.render_widget(end_field, chunks[1]);

    // Set cursor position
    let inner = if app.log_filter_panel == LogFilterPanel::StartDate {
        chunks[0].inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 })
    } else {
        chunks[1].inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 })
    };
    let input = if app.log_filter_panel == LogFilterPanel::StartDate {
        &app.start_date_input
    } else {
        &app.end_date_input
    };
    frame.set_cursor_position((
        inner.x + input.len() as u16,
        inner.y,
    ));
}

fn render_project_filter_popup(frame: &mut Frame, app: &App, area: Rect) {
    let popup_width = 50u16;
    let popup_height = 15u16.min(area.height.saturating_sub(4));
    let popup_x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let all_projects = app.all_project_names();
    let items: Vec<ListItem> = all_projects
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let selected = app.log_filter.projects.contains(name);
            let checkbox = if selected { "[x]" } else { "[ ]" };
            let style = if i == app.log_filter_project_selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else if selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(checkbox, if selected { Style::default().fg(Color::Green) } else { Style::default() }),
                Span::raw(" "),
                Span::styled(name.as_str(), style),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.log_filter_project_selected));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Projects (↑↓ navigate, x toggle, ESC close)")
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(list, popup_area, &mut state);
}

fn render_people_filter_popup(frame: &mut Frame, app: &App, area: Rect) {
    let popup_width = 50u16;
    let popup_height = 15u16.min(area.height.saturating_sub(4));
    let popup_x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let all_people = app.all_people_names();
    let items: Vec<ListItem> = all_people
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let selected = app.log_filter.people.contains(name);
            let checkbox = if selected { "[x]" } else { "[ ]" };
            let style = if i == app.log_filter_people_selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else if selected {
                Style::default().fg(Color::Blue)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(checkbox, if selected { Style::default().fg(Color::Blue) } else { Style::default() }),
                Span::raw(" "),
                Span::styled(name.as_str(), style),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.log_filter_people_selected));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("People (↑↓ navigate, x toggle, ESC close)")
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(list, popup_area, &mut state);
}

fn render_log_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .filtered_logs
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

            let style = if i == app.log_selected {
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
    state.select(Some(app.log_selected));

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Log Entries"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(list, area, &mut state);
}

pub fn render_view_log(frame: &mut Frame, app: &App, area: Rect, path: &std::path::PathBuf) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Content
            Constraint::Length(3),  // Help bar
        ])
        .split(area);

    // Title
    let title_text = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("Log Entry");
    let title = Paragraph::new(format!("Viewing: {}", title_text))
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Content
    let content = if let Ok(text) = std::fs::read_to_string(path) {
        text
    } else {
        "Error reading log file".to_string()
    };

    // Syntax highlight the content
    let lines: Vec<Line> = content
        .lines()
        .map(|line| {
            let mut spans: Vec<Span> = Vec::new();
            let mut current_word = String::new();

            for c in line.chars() {
                if c.is_whitespace() {
                    if !current_word.is_empty() {
                        let style = word_style(&current_word);
                        spans.push(Span::styled(current_word.clone(), style));
                        current_word.clear();
                    }
                    spans.push(Span::raw(c.to_string()));
                } else {
                    current_word.push(c);
                }
            }

            if !current_word.is_empty() {
                let style = word_style(&current_word);
                spans.push(Span::styled(current_word, style));
            }

            Line::from(spans)
        })
        .collect();

    let log_content = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Content"))
        .wrap(Wrap { trim: false })
        .scroll((app.view_log_scroll, 0));
    frame.render_widget(log_content, chunks[1]);

    // Help bar
    let help_text = vec![
        Span::styled("↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(" Scroll  "),
        Span::styled("ESC", Style::default().fg(Color::Yellow)),
        Span::raw(" Back"),
    ];
    let help = Paragraph::new(Line::from(help_text))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[2]);
}

fn word_style(word: &str) -> Style {
    if word.starts_with('#') {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else if word.starts_with('@') {
        Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
    } else if word.starts_with("[]") {
        Style::default().fg(Color::Magenta)
    } else if word.starts_with("[x]") || word.starts_with("[X]") {
        Style::default().fg(Color::Gray).add_modifier(Modifier::CROSSED_OUT)
    } else {
        Style::default()
    }
}
