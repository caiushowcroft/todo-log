use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
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

    // Title - show different text for new vs edit
    let title_text = match app.screen {
        super::app::Screen::ProjectEdit(None) => "New Project",
        _ => "Edit Project",
    };
    let title = Paragraph::new(title_text)
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

    // Status field (field 3) - dropdown
    let status_style = if app.project_edit_field == 3 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let status_display = if app.project_edit_status.is_empty() {
        "(select status)".to_string()
    } else {
        app.project_edit_status.clone()
    };
    let status_text = format!("{} {}", status_display, if app.project_edit_field == 3 { "▼" } else { "" });
    let status_field = Paragraph::new(status_text)
        .style(status_style)
        .block(Block::default().borders(Borders::ALL).title("[4] Status (↑↓ to select, Enter to confirm)"));
    frame.render_widget(status_field, chunks[4]);

    // Group field (field 4) - dropdown
    let group_style = if app.project_edit_field == 4 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let group_display = if app.project_edit_group.is_empty() {
        "(no group)".to_string()
    } else {
        app.project_edit_group.clone()
    };
    let group_text = format!("{} {}", group_display, if app.project_edit_field == 4 { "▼" } else { "" });
    let group_field = Paragraph::new(group_text)
        .style(group_style)
        .block(Block::default().borders(Borders::ALL).title("[5] Group (↑↓ to select, Enter to confirm)"));
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

    // Set cursor position for active field (only for text input fields, not dropdowns)
    if app.project_edit_field < 3 {
        let (field_area, input) = match app.project_edit_field {
            0 => (chunks[1], &app.project_edit_name),
            1 => (chunks[2], &app.project_edit_description),
            2 => (chunks[3], &app.project_edit_jira),
            _ => (chunks[1], &app.project_edit_name),
        };

        let inner = field_area.inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 });
        frame.set_cursor_position((
            inner.x + input.len() as u16,
            inner.y,
        ));
    }

    // Render dropdowns if open
    if app.project_edit_field == 3 && app.project_edit_status_dropdown_open {
        render_status_dropdown(frame, app, chunks[4]);
    }
    if app.project_edit_field == 4 && app.project_edit_group_dropdown_open {
        render_group_dropdown(frame, app, chunks[5]);
    }
}

fn render_status_dropdown(frame: &mut Frame, app: &App, field_area: Rect) {
    let states = app.config.allowed_state_names();
    if states.is_empty() {
        return;
    }

    // Calculate dropdown position (below the field)
    let dropdown_height = (states.len() as u16 + 2).min(10); // +2 for borders, max 10
    let dropdown_width = field_area.width;
    let dropdown_x = field_area.x;
    let dropdown_y = field_area.y + field_area.height;

    // Make sure dropdown fits on screen
    let dropdown_area = Rect::new(dropdown_x, dropdown_y, dropdown_width, dropdown_height);

    frame.render_widget(Clear, dropdown_area);

    let items: Vec<ListItem> = states
        .iter()
        .enumerate()
        .map(|(i, state)| {
            let state_color = app.config.get_state_color(state);
            let style = if i == app.project_edit_status_dropdown_selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default().fg(state_color)
            };
            ListItem::new(Line::from(Span::styled(state.as_str(), style)))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.project_edit_status_dropdown_selected));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(list, dropdown_area, &mut state);
}

fn render_group_dropdown(frame: &mut Frame, app: &App, field_area: Rect) {
    let mut groups = app.config.allowed_groups();
    // Add "(no group)" option at the beginning
    groups.insert(0, "(no group)".to_string());

    if groups.is_empty() {
        return;
    }

    // Calculate dropdown position (below the field)
    let dropdown_height = (groups.len() as u16 + 2).min(10); // +2 for borders, max 10
    let dropdown_width = field_area.width;
    let dropdown_x = field_area.x;
    let dropdown_y = field_area.y + field_area.height;

    // Make sure dropdown fits on screen
    let dropdown_area = Rect::new(dropdown_x, dropdown_y, dropdown_width, dropdown_height);

    frame.render_widget(Clear, dropdown_area);

    let items: Vec<ListItem> = groups
        .iter()
        .enumerate()
        .map(|(i, group)| {
            let style = if i == app.project_edit_group_dropdown_selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(Span::styled(group.as_str(), style)))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.project_edit_group_dropdown_selected));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(list, dropdown_area, &mut state);
}
