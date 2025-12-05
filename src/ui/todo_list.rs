use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use super::app::{App, TodoFilterPanel};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(5),  // Filters summary
            Constraint::Min(10),    // Todo list
            Constraint::Length(3),  // Help bar
        ])
        .split(area);

    // Title
    let title = Paragraph::new(format!("Todos ({} items)", app.filtered_todos.len()))
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Filters summary
    render_filters_summary(frame, app, chunks[1]);

    // Todo list
    render_todo_list(frame, app, chunks[2]);

    // Help bar
    let help_text = if app.todo_filter_panel == TodoFilterPanel::None {
        vec![
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::raw(" Navigate  "),
            Span::styled("x", Style::default().fg(Color::Yellow)),
            Span::raw(" Toggle  "),
            Span::styled("l", Style::default().fg(Color::Yellow)),
            Span::raw(" View log  "),
            Span::styled("c", Style::default().fg(Color::Yellow)),
            Span::raw(" Completed  "),
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
    match app.todo_filter_panel {
        TodoFilterPanel::None => {}
        TodoFilterPanel::Completed => {
            render_completed_filter_popup(frame, app, area);
        }
        TodoFilterPanel::Projects => {
            render_project_filter_popup(frame, app, area);
        }
        TodoFilterPanel::People => {
            render_people_filter_popup(frame, app, area);
        }
    }
}

fn render_filters_summary(frame: &mut Frame, app: &App, area: Rect) {
    let filter_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),  // Completed
            Constraint::Percentage(33),  // Projects
            Constraint::Percentage(34),  // People
        ])
        .split(area);

    // Completed filter summary
    let completed_text = if app.todo_filter.show_completed {
        "[c] Show: All"
    } else {
        "[c] Show: Open only"
    };
    let completed_style = if app.todo_filter_panel == TodoFilterPanel::Completed {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let completed = Paragraph::new(completed_text)
        .style(completed_style)
        .block(Block::default().borders(Borders::ALL).title("Status"));
    frame.render_widget(completed, filter_chunks[0]);

    // Projects filter summary
    let projects_text = if app.todo_filter.projects.is_empty() {
        "[p] All".to_string()
    } else {
        format!("[p] {}", app.todo_filter.projects.join(", "))
    };
    let projects_style = if app.todo_filter_panel == TodoFilterPanel::Projects {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let projects = Paragraph::new(projects_text)
        .style(projects_style)
        .block(Block::default().borders(Borders::ALL).title("Projects"));
    frame.render_widget(projects, filter_chunks[1]);

    // People filter summary
    let people_text = if app.todo_filter.people.is_empty() {
        "[h] All".to_string()
    } else {
        format!("[h] {}", app.todo_filter.people.join(", "))
    };
    let people_style = if app.todo_filter_panel == TodoFilterPanel::People {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let people = Paragraph::new(people_text)
        .style(people_style)
        .block(Block::default().borders(Borders::ALL).title("People"));
    frame.render_widget(people, filter_chunks[2]);
}

fn render_completed_filter_popup(frame: &mut Frame, app: &App, area: Rect) {
    let popup_width = 35u16;
    let popup_height = 6u16;
    let popup_x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let items = vec![
        ListItem::new(Line::from(vec![
            Span::styled(
                if !app.todo_filter.show_completed { "[x]" } else { "[ ]" },
                if !app.todo_filter.show_completed { Style::default().fg(Color::Green) } else { Style::default() },
            ),
            Span::raw(" Show open todos only"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                if app.todo_filter.show_completed { "[x]" } else { "[ ]" },
                if app.todo_filter.show_completed { Style::default().fg(Color::Green) } else { Style::default() },
            ),
            Span::raw(" Show all todos"),
        ])),
    ];

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Status Filter (x toggle, ESC close)")
                .border_style(Style::default().fg(Color::Yellow)),
        );

    frame.render_widget(list, popup_area);
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
            let selected = app.todo_filter.projects.contains(name);
            let checkbox = if selected { "[x]" } else { "[ ]" };
            let style = if i == app.todo_filter_project_selected {
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
    state.select(Some(app.todo_filter_project_selected));

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
            let selected = app.todo_filter.people.contains(name);
            let checkbox = if selected { "[x]" } else { "[ ]" };
            let style = if i == app.todo_filter_people_selected {
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
    state.select(Some(app.todo_filter_people_selected));

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

fn render_todo_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .filtered_todos
        .iter()
        .enumerate()
        .map(|(i, todo)| {
            let checkbox = if todo.completed { "[x]" } else { "[]" };

            let is_selected = i == app.todo_selected;
            let text_style = if is_selected {
                if todo.completed {
                    Style::default()
                        .bg(Color::DarkGray)
                        .fg(Color::Gray)
                        .add_modifier(Modifier::CROSSED_OUT)
                } else {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                }
            } else if todo.completed {
                Style::default().fg(Color::Gray).add_modifier(Modifier::CROSSED_OUT)
            } else {
                Style::default()
            };

            // Build spans with proper formatting
            let mut spans = vec![
                Span::styled(
                    checkbox,
                    if todo.completed {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Yellow)
                    },
                ),
                Span::raw(" "),
                Span::styled(&todo.text, text_style),
            ];

            // Add project tags
            for project in &todo.projects {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    format!("#{}", project),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ));
            }

            // Add people tags
            for person in &todo.people {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    format!("@{}", person),
                    Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.todo_selected));

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Todos"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(list, area, &mut state);
}
