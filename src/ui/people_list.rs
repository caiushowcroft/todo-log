use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use super::app::App;

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // People list
            Constraint::Length(3),  // Help bar
        ])
        .split(area);

    // Title
    let title = Paragraph::new(format!("People ({} total)", app.people.len()))
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // People list
    render_people_list(frame, app, chunks[1]);

    // Help bar
    let help_text = vec![
        Span::styled("↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(" Navigate  "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" View  "),
        Span::styled("n", Style::default().fg(Color::Yellow)),
        Span::raw(" New  "),
        Span::styled("ESC", Style::default().fg(Color::Yellow)),
        Span::raw(" Back"),
    ];
    let help = Paragraph::new(Line::from(help_text))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[2]);
}

fn render_people_list(frame: &mut Frame, app: &mut App, area: Rect) {
    // Calculate available width for fields
    let available_width = area.width.saturating_sub(2); // Account for block borders
    let fixed_overhead = 10; // bullet + spacing + padding

    // Find the longest name to determine name column width
    let max_name_len = app.people.iter()
        .map(|p| p.name.len())
        .max()
        .unwrap_or(15)
        .min(20); // Cap at 20 chars

    // Calculate full_name width
    let full_name_width = available_width
        .saturating_sub(fixed_overhead)
        .saturating_sub(max_name_len as u16)
        .max(30) as usize; // Minimum 30 chars for full name

    let mut items: Vec<ListItem> = Vec::new();
    let visual_items_before_scroll = app.person_list_scroll;

    for (i, person) in app.people.iter().enumerate() {
        let is_selected = i == app.person_selected;
        let text_style = if is_selected {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default()
        };

        // Format name to fixed width
        let name_formatted = format!("{:width$}", person.name, width = max_name_len);

        // Format full name
        let full_name_raw = person.full_name.as_ref()
            .map(|f| f.as_str())
            .unwrap_or("(no full name)");

        // Truncate or pad full name to exact width
        let full_name: String = if full_name_raw.len() > full_name_width {
            format!("{:width$}", full_name_raw.chars().take(full_name_width - 3).collect::<String>() + "...", width = full_name_width)
        } else {
            format!("{:width$}", full_name_raw, width = full_name_width)
        };

        items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("• {}", name_formatted), text_style.add_modifier(Modifier::BOLD)),
            Span::raw(" - "),
            Span::styled(full_name, text_style),
        ])));
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("People"));

    // Use ListState to handle scrolling
    let mut state = ListState::default();
    state.select(None); // We don't use state selection since we have manual styling
    if visual_items_before_scroll > 0 {
        *state.offset_mut() = visual_items_before_scroll;
    }

    frame.render_stateful_widget(list, area, &mut state);
}
