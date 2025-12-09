use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use super::app::{App, ProjectFilterPanel};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(3),  // Filter summary
            Constraint::Min(10),    // Project list
            Constraint::Length(3),  // Help bar
        ])
        .split(area);

    // Title
    let title = Paragraph::new(format!("Projects ({} total)", app.filtered_projects.len()))
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Filter summary
    render_filter_summary(frame, app, chunks[1]);

    // Project list
    render_project_list(frame, app, chunks[2]);

    // Help bar
    let help_text = if app.project_filter_panel == ProjectFilterPanel::None {
        vec![
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::raw(" Navigate  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" Edit  "),
            Span::styled("g", Style::default().fg(Color::Yellow)),
            Span::raw(" Filter groups  "),
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
    if app.project_filter_panel == ProjectFilterPanel::Groups {
        render_group_filter_popup(frame, app, area);
    }
}

fn render_filter_summary(frame: &mut Frame, app: &App, area: Rect) {
    let groups_text = if app.project_filter_groups.is_empty() {
        "[g] All groups".to_string()
    } else {
        format!("[g] Groups: {}", app.project_filter_groups.join(", "))
    };
    let groups_style = if app.project_filter_panel == ProjectFilterPanel::Groups {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let groups = Paragraph::new(groups_text)
        .style(groups_style)
        .block(Block::default().borders(Borders::ALL).title("Filter"));
    frame.render_widget(groups, area);
}

fn render_group_filter_popup(frame: &mut Frame, app: &App, area: Rect) {
    let popup_width = 50u16;
    let popup_height = 15u16.min(area.height.saturating_sub(4));
    let popup_x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let all_groups = app.all_group_names();
    let items: Vec<ListItem> = all_groups
        .iter()
        .enumerate()
        .map(|(i, group_name)| {
            // Convert "(No group)" to empty string for comparison
            let group_for_check = if group_name == "(No group)" { "" } else { group_name.as_str() };
            let selected = app.project_filter_groups.contains(&group_for_check.to_string());
            let checkbox = if selected { "[x]" } else { "[ ]" };
            let style = if i == app.project_filter_group_selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else if selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(checkbox, if selected { Style::default().fg(Color::Green) } else { Style::default() }),
                Span::raw(" "),
                Span::styled(group_name.as_str(), style),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.project_filter_group_selected));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Groups (↑↓ navigate, x toggle, ESC close)")
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(list, popup_area, &mut state);
}

fn render_project_list(frame: &mut Frame, app: &mut App, area: Rect) {
    // Group projects by group field
    let mut grouped_projects: Vec<(String, Vec<&crate::models::Project>)> = Vec::new();

    for project in &app.filtered_projects {
        let group_name = if project.group.is_empty() {
            "(No group)".to_string()
        } else {
            project.group.clone()
        };

        if let Some((_, projects)) = grouped_projects.iter_mut().find(|(g, _)| g == &group_name) {
            projects.push(project);
        } else {
            grouped_projects.push((group_name, vec![project]));
        }
    }

    // Sort groups alphabetically, with "(No group)" last
    grouped_projects.sort_by(|(a, _), (b, _)| {
        if a == "(No group)" {
            std::cmp::Ordering::Greater
        } else if b == "(No group)" {
            std::cmp::Ordering::Less
        } else {
            a.cmp(b)
        }
    });

    // Calculate available width for description
    // Account for: indent(2) + bullet(2) + separator(3) + space(1) + status(10) + borders(2) + padding(2)
    let available_width = area.width.saturating_sub(2); // Account for block borders
    let fixed_overhead = 20; // indent + bullet + separator + space + status + padding

    // Find the longest project name to determine name column width
    let max_name_len = app.filtered_projects.iter()
        .map(|p| p.name.len())
        .max()
        .unwrap_or(20)
        .min(30); // Cap at 30 chars

    // Calculate description width
    let desc_width = available_width
        .saturating_sub(fixed_overhead)
        .saturating_sub(max_name_len as u16)
        .max(30) as usize; // Minimum 30 chars for description

    let mut items: Vec<ListItem> = Vec::new();
    let mut project_index = 0;

    for (group_name, projects) in &grouped_projects {
        // Add group header
        items.push(ListItem::new(Line::from(vec![
            Span::styled(
                format!("▼ {}", group_name),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ])));

        // Add projects in this group
        for project in projects {
            let is_selected = project_index == app.project_selected;
            let text_style = if is_selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            // Format description to fixed width
            let desc_raw = project.description.as_ref()
                .map(|d| d.as_str())
                .unwrap_or("(no description)");

            // Truncate or pad description to exact width
            let desc: String = if desc_raw.len() > desc_width {
                format!("{:width$}", desc_raw.chars().take(desc_width - 3).collect::<String>() + "...", width = desc_width)
            } else {
                format!("{:width$}", desc_raw, width = desc_width)
            };

            let status_color = match project.status.as_str() {
                "open" => Color::Green,
                "closed" => Color::Gray,
                _ => Color::Yellow,
            };

            // Format project name to fixed width
            let name_formatted = format!("{:width$}", project.name, width = max_name_len);

            items.push(ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("• {}", name_formatted), text_style.add_modifier(Modifier::BOLD)),
                Span::raw(" - "),
                Span::styled(desc, text_style),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", project.status),
                    Style::default().fg(status_color),
                ),
            ])));
            project_index += 1;
        }
    }

    // We're using manual styling for selection, so we don't need the List's built-in highlighting
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Project List"));

    frame.render_widget(list, area);
}
