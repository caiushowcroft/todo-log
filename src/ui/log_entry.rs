use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use super::app::{App, AutocompleteType};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Header with timestamp
            Constraint::Min(10),    // Editor area
            Constraint::Length(5),  // Attachments
            Constraint::Length(3),  // Help/status bar
        ])
        .split(area);

    // Header with timestamp
    let header_content = if app.timestamp_editing {
        format!("Edit Timestamp: {}", app.timestamp_edit_input)
    } else {
        let timestamp = app.current_log.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
        format!("New Log Entry - {}", timestamp)
    };
    let header_style = if app.timestamp_editing {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    };
    let header = Paragraph::new(header_content)
        .style(header_style)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);

    // Editor area
    render_editor(frame, app, chunks[1]);

    // Attachments
    render_attachments(frame, app, chunks[2]);

    // Help bar
    let help_text = if app.file_browser_open {
        vec![
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::raw(" Navigate  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" Select  "),
            Span::styled("Backspace", Style::default().fg(Color::Yellow)),
            Span::raw(" Parent dir  "),
            Span::styled("ESC", Style::default().fg(Color::Yellow)),
            Span::raw(" Cancel"),
        ]
    } else if app.timestamp_editing {
        vec![
            Span::styled("←→", Style::default().fg(Color::Yellow)),
            Span::raw(" Move  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" Apply  "),
            Span::styled("ESC", Style::default().fg(Color::Yellow)),
            Span::raw(" Cancel  "),
            Span::styled("Format:", Style::default().fg(Color::Cyan)),
            Span::raw(" YYYY-MM-DD HH:MM:SS"),
        ]
    } else {
        vec![
            Span::styled("Ctrl+S", Style::default().fg(Color::Yellow)),
            Span::raw(" Save  "),
            Span::styled("Ctrl+A", Style::default().fg(Color::Yellow)),
            Span::raw(" Attach  "),
            Span::styled("Ctrl+T", Style::default().fg(Color::Yellow)),
            Span::raw(" Edit time  "),
            Span::styled("ESC", Style::default().fg(Color::Yellow)),
            Span::raw(" Cancel  "),
            Span::styled("#", Style::default().fg(Color::Green)),
            Span::raw("project  "),
            Span::styled("@", Style::default().fg(Color::Blue)),
            Span::raw("person  "),
            Span::styled("[]", Style::default().fg(Color::Magenta)),
            Span::raw(" todo"),
        ]
    };
    let help = Paragraph::new(Line::from(help_text))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[3]);

    // Render autocomplete popup if active (and file browser not open)
    if !app.file_browser_open && app.autocomplete_active {
        render_autocomplete(frame, app, chunks[1]);
    }

    // Render file browser popup if open
    if app.file_browser_open {
        render_file_browser(frame, app, area);
    }

    // Set cursor position for timestamp editing
    if app.timestamp_editing {
        let inner = chunks[0].inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 });
        let prompt_len = "Edit Timestamp: ".len() as u16;
        frame.set_cursor_position((
            inner.x + prompt_len + app.timestamp_edit_cursor as u16,
            inner.y,
        ));
    }
}

fn render_attachments(frame: &mut Frame, app: &App, area: Rect) {
    if app.attachments.is_empty() {
        let attachments = Paragraph::new("No attachments (Ctrl+A to add)")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL).title("Attachments"));
        frame.render_widget(attachments, area);
    } else {
        let items: Vec<ListItem> = app
            .attachments
            .iter()
            .map(|path| {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.to_string_lossy().to_string());

                // Get file size if available
                let size_str = std::fs::metadata(path)
                    .map(|m| format_size(m.len()))
                    .unwrap_or_default();

                ListItem::new(Line::from(vec![
                    Span::styled("📎 ", Style::default()),
                    Span::styled(name, Style::default().fg(Color::Cyan)),
                    Span::styled(format!(" ({})", size_str), Style::default().fg(Color::DarkGray)),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(format!("Attachments ({})", app.attachments.len())));
        frame.render_widget(list, area);
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn render_file_browser(frame: &mut Frame, app: &App, area: Rect) {
    let popup_width = 60u16.min(area.width.saturating_sub(4));
    let popup_height = 20u16.min(area.height.saturating_sub(4));
    let popup_x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),  // Current path
            Constraint::Min(5),     // File list
        ])
        .split(popup_area);

    // Outer border
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title("Select File (↑↓ navigate, Enter select, Backspace parent)")
        .border_style(Style::default().fg(Color::Yellow));
    frame.render_widget(outer_block, popup_area);

    // Current path
    let path_str = app.file_browser_dir.to_string_lossy();
    let path_display = Paragraph::new(path_str.as_ref())
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(path_display, chunks[0]);

    // File list
    let items: Vec<ListItem> = app
        .file_browser_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let icon = if entry.is_dir { "📁 " } else { "📄 " };
            let size_str = if entry.is_dir {
                String::new()
            } else {
                format!(" ({})", format_size(entry.size))
            };

            let style = if i == app.file_browser_selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else if entry.is_dir {
                Style::default().fg(Color::Blue)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![
                Span::raw(icon),
                Span::styled(&entry.name, style),
                Span::styled(size_str, Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.file_browser_selected));

    let list = List::new(items)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(list, chunks[1], &mut state);
}

fn render_editor(frame: &mut Frame, app: &App, area: Rect) {
    let content = &app.current_log.content;

    // Process content line by line for proper multi-line support
    let lines: Vec<Line> = content
        .split('\n')
        .map(|line_str| {
            let mut spans: Vec<Span> = Vec::new();
            let mut current_word = String::new();

            for c in line_str.chars() {
                if c.is_whitespace() {
                    // Flush current word
                    if !current_word.is_empty() {
                        let style = get_word_style(&current_word);
                        spans.push(Span::styled(current_word.clone(), style));
                        current_word.clear();
                    }
                    spans.push(Span::raw(c.to_string()));
                } else {
                    current_word.push(c);
                }
            }

            // Flush remaining word
            if !current_word.is_empty() {
                let style = get_word_style(&current_word);
                spans.push(Span::styled(current_word, style));
            }

            Line::from(spans)
        })
        .collect();

    let editor = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Log Content (type your log entry)")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(editor, area);

    // Only show cursor if file browser is not open
    if !app.file_browser_open {
        // Calculate cursor position accounting for newlines
        let inner = area.inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 });

        // Find which line and column the cursor is on
        let before_cursor: String = content.chars().take(app.log_cursor_pos).collect();
        let lines_before: Vec<&str> = before_cursor.split('\n').collect();
        let cursor_y = (lines_before.len() - 1) as u16;
        let cursor_x = lines_before.last().map(|l| l.chars().count()).unwrap_or(0) as u16;

        frame.set_cursor_position((
            inner.x + cursor_x.min(inner.width.saturating_sub(1)),
            inner.y + cursor_y.min(inner.height.saturating_sub(1)),
        ));
    }
}

fn get_word_style(word: &str) -> Style {
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

fn render_autocomplete(frame: &mut Frame, app: &App, editor_area: Rect) {
    if app.autocomplete_suggestions.is_empty() {
        return;
    }

    // Calculate popup position (below cursor, roughly)
    let popup_height = (app.autocomplete_suggestions.len() + 2).min(10) as u16;
    let popup_width = 30u16;

    // Position near cursor
    let inner = editor_area.inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 });
    let cursor_x = (app.log_cursor_pos % (inner.width as usize)) as u16;
    let cursor_y = (app.log_cursor_pos / (inner.width as usize)) as u16;

    let popup_x = (inner.x + cursor_x).min(editor_area.right().saturating_sub(popup_width));
    let popup_y = (inner.y + cursor_y + 1).min(editor_area.bottom().saturating_sub(popup_height));

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear the area and render popup
    frame.render_widget(Clear, popup_area);

    let title = match app.autocomplete_type {
        AutocompleteType::Project => "Projects",
        AutocompleteType::Person => "People",
        AutocompleteType::None => "Suggestions",
    };

    let items: Vec<ListItem> = app
        .autocomplete_suggestions
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if i == app.autocomplete_index {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(s.as_str()).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Yellow)),
        );

    frame.render_widget(list, popup_area);
}
