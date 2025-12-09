mod models;
mod storage;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use ui::app::{App, LogFilterPanel, Screen, TodoFilterPanel};

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new()?;
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| {
            let area = f.area();
            match &app.screen {
                Screen::Menu => ui::menu::render(f, app, area),
                Screen::LogEntry => ui::log_entry::render(f, app, area),
                Screen::TodoList => ui::todo_list::render(f, app, area),
                Screen::LogList => ui::log_list::render(f, app, area),
                Screen::ViewLog(path) => ui::log_list::render_view_log(f, app, area, path),
                Screen::ProjectList => ui::project_list::render(f, app, area),
                Screen::ProjectDetails(idx) => ui::project_details::render(f, app, area, *idx),
                Screen::ProjectEdit(_) => ui::project_edit::render(f, app, area),
            }
        })?;

        if !app.running {
            return Ok(());
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.clear_status();

                match &app.screen {
                    Screen::Menu => handle_menu_input(app, key.code),
                    Screen::LogEntry => handle_log_entry_input(app, key.code, key.modifiers)?,
                    Screen::TodoList => handle_todo_list_input(app, key.code)?,
                    Screen::LogList => handle_log_list_input(app, key.code),
                    Screen::ViewLog(_) => handle_view_log_input(app, key.code),
                    Screen::ProjectList => handle_project_list_input(app, key.code)?,
                    Screen::ProjectDetails(_) => handle_project_details_input(app, key.code)?,
                    Screen::ProjectEdit(_) => handle_project_edit_input(app, key.code, key.modifiers)?,
                }
            }
        }
    }
}

fn handle_menu_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Up => {
            if app.menu_selected > 0 {
                app.menu_selected -= 1;
            }
        }
        KeyCode::Down => {
            if app.menu_selected < 3 {
                app.menu_selected += 1;
            }
        }
        KeyCode::Enter => {
            execute_menu_selection(app, app.menu_selected);
        }
        KeyCode::Char('1') => {
            app.menu_selected = 0;
            execute_menu_selection(app, 0);
        }
        KeyCode::Char('2') => {
            app.menu_selected = 1;
            execute_menu_selection(app, 1);
        }
        KeyCode::Char('3') => {
            app.menu_selected = 2;
            execute_menu_selection(app, 2);
        }
        KeyCode::Char('4') => {
            app.menu_selected = 3;
            execute_menu_selection(app, 3);
        }
        KeyCode::Esc => app.quit(),
        _ => {}
    }
}

fn execute_menu_selection(app: &mut App, selection: usize) {
    match selection {
        0 => app.start_new_log(),
        1 => {
            let _ = app.show_todos();
        }
        2 => {
            let _ = app.show_logs();
        }
        3 => {
            let _ = app.show_projects();
        }
        _ => {}
    }
}

fn handle_log_entry_input(app: &mut App, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
    // Handle file browser input first if open
    if app.file_browser_open {
        match key {
            KeyCode::Esc => {
                app.close_file_browser();
            }
            KeyCode::Up => {
                if app.file_browser_selected > 0 {
                    app.file_browser_selected -= 1;
                }
            }
            KeyCode::Down => {
                if app.file_browser_selected < app.file_browser_entries.len().saturating_sub(1) {
                    app.file_browser_selected += 1;
                }
            }
            KeyCode::Enter => {
                app.file_browser_enter();
            }
            KeyCode::Backspace => {
                app.file_browser_go_up();
            }
            _ => {}
        }
        return Ok(());
    }

    // Handle autocomplete navigation
    if app.autocomplete_active {
        match key {
            KeyCode::Down => {
                if app.autocomplete_index < app.autocomplete_suggestions.len().saturating_sub(1) {
                    app.autocomplete_index += 1;
                }
                return Ok(());
            }
            KeyCode::Up => {
                if app.autocomplete_index > 0 {
                    app.autocomplete_index -= 1;
                }
                return Ok(());
            }
            KeyCode::Tab | KeyCode::Enter => {
                app.accept_autocomplete();
                return Ok(());
            }
            KeyCode::Esc => {
                app.autocomplete_active = false;
                return Ok(());
            }
            _ => {}
        }
    }

    // Check for Ctrl combinations
    if modifiers.contains(KeyModifiers::CONTROL) {
        match key {
            KeyCode::Char('s') => {
                app.save_log()?;
                return Ok(());
            }
            KeyCode::Char('a') => {
                // Open file browser
                app.open_file_browser();
                return Ok(());
            }
            _ => {}
        }
    }

    match key {
        KeyCode::Esc => {
            app.go_to_screen(Screen::Menu);
        }
        KeyCode::Char(c) => {
            app.insert_char(c);
        }
        KeyCode::Enter => {
            app.insert_char('\n');
        }
        KeyCode::Backspace => {
            app.delete_char();
        }
        KeyCode::Left => {
            app.move_cursor_left();
        }
        KeyCode::Right => {
            app.move_cursor_right();
        }
        KeyCode::Up => {
            app.move_cursor_up();
        }
        KeyCode::Down => {
            app.move_cursor_down();
        }
        KeyCode::Home => {
            app.log_cursor_pos = 0;
        }
        KeyCode::End => {
            app.log_cursor_pos = app.current_log.content.chars().count();
        }
        _ => {}
    }

    Ok(())
}

fn handle_todo_list_input(app: &mut App, key: KeyCode) -> Result<()> {
    // Handle filter panel input if one is open
    match app.todo_filter_panel {
        TodoFilterPanel::Completed => {
            handle_todo_completed_filter_input(app, key);
            return Ok(());
        }
        TodoFilterPanel::Projects => {
            handle_todo_project_filter_input(app, key);
            return Ok(());
        }
        TodoFilterPanel::People => {
            handle_todo_people_filter_input(app, key);
            return Ok(());
        }
        TodoFilterPanel::None => {}
    }

    // Normal todo list navigation
    match key {
        KeyCode::Esc => {
            app.go_to_screen(Screen::Menu);
        }
        KeyCode::Up => {
            if app.todo_selected > 0 {
                app.todo_selected -= 1;
            }
        }
        KeyCode::Down => {
            if app.todo_selected < app.filtered_todos.len().saturating_sub(1) {
                app.todo_selected += 1;
            }
        }
        KeyCode::Char('x') => {
            app.toggle_selected_todo()?;
            app.apply_todo_filter();
        }
        KeyCode::Char('l') => {
            app.view_todo_log();
        }
        KeyCode::Char('c') => {
            // Open completed filter
            app.todo_filter_panel = TodoFilterPanel::Completed;
        }
        KeyCode::Char('p') => {
            // Open projects filter
            app.todo_filter_project_selected = 0;
            app.todo_filter_panel = TodoFilterPanel::Projects;
        }
        KeyCode::Char('h') => {
            // Open people filter
            app.todo_filter_people_selected = 0;
            app.todo_filter_panel = TodoFilterPanel::People;
        }
        _ => {}
    }

    Ok(())
}

fn handle_todo_completed_filter_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => {
            app.todo_filter_panel = TodoFilterPanel::None;
        }
        KeyCode::Char('x') => {
            app.todo_filter.show_completed = !app.todo_filter.show_completed;
            app.apply_todo_filter();
            app.todo_selected = 0;
        }
        _ => {}
    }
}

fn handle_todo_project_filter_input(app: &mut App, key: KeyCode) {
    let all_projects = app.all_project_names();

    match key {
        KeyCode::Esc => {
            app.todo_filter_panel = TodoFilterPanel::None;
            app.todo_selected = 0;
        }
        KeyCode::Up => {
            if app.todo_filter_project_selected > 0 {
                app.todo_filter_project_selected -= 1;
            }
        }
        KeyCode::Down => {
            if app.todo_filter_project_selected < all_projects.len().saturating_sub(1) {
                app.todo_filter_project_selected += 1;
            }
        }
        KeyCode::Char('x') => {
            // Toggle the selected project
            if let Some(project) = all_projects.get(app.todo_filter_project_selected) {
                app.toggle_todo_filter_project(project);
            }
            app.todo_selected = 0;
        }
        _ => {}
    }
}

fn handle_todo_people_filter_input(app: &mut App, key: KeyCode) {
    let all_people = app.all_people_names();

    match key {
        KeyCode::Esc => {
            app.todo_filter_panel = TodoFilterPanel::None;
            app.todo_selected = 0;
        }
        KeyCode::Up => {
            if app.todo_filter_people_selected > 0 {
                app.todo_filter_people_selected -= 1;
            }
        }
        KeyCode::Down => {
            if app.todo_filter_people_selected < all_people.len().saturating_sub(1) {
                app.todo_filter_people_selected += 1;
            }
        }
        KeyCode::Char('x') => {
            // Toggle the selected person
            if let Some(person) = all_people.get(app.todo_filter_people_selected) {
                app.toggle_todo_filter_person(person);
            }
            app.todo_selected = 0;
        }
        _ => {}
    }
}

fn handle_log_list_input(app: &mut App, key: KeyCode) {
    // Handle filter panel input if one is open
    match app.log_filter_panel {
        LogFilterPanel::StartDate => {
            handle_date_filter_input(app, key, true);
            return;
        }
        LogFilterPanel::EndDate => {
            handle_date_filter_input(app, key, false);
            return;
        }
        LogFilterPanel::Projects => {
            handle_project_filter_input(app, key);
            return;
        }
        LogFilterPanel::People => {
            handle_people_filter_input(app, key);
            return;
        }
        LogFilterPanel::None => {}
    }

    // Normal log list navigation
    match key {
        KeyCode::Esc => {
            app.go_to_screen(Screen::Menu);
        }
        KeyCode::Up => {
            if app.log_selected > 0 {
                app.log_selected -= 1;
            }
        }
        KeyCode::Down => {
            if app.log_selected < app.filtered_logs.len().saturating_sub(1) {
                app.log_selected += 1;
            }
        }
        KeyCode::Char('l') | KeyCode::Enter => {
            app.view_selected_log();
        }
        KeyCode::Char('s') => {
            // Open start date filter
            app.init_date_inputs();
            app.log_filter_panel = LogFilterPanel::StartDate;
        }
        KeyCode::Char('e') => {
            // Open end date filter
            app.init_date_inputs();
            app.log_filter_panel = LogFilterPanel::EndDate;
        }
        KeyCode::Char('p') => {
            // Open projects filter
            app.log_filter_project_selected = 0;
            app.log_filter_panel = LogFilterPanel::Projects;
        }
        KeyCode::Char('h') => {
            // Open people filter
            app.log_filter_people_selected = 0;
            app.log_filter_panel = LogFilterPanel::People;
        }
        _ => {}
    }
}

fn handle_date_filter_input(app: &mut App, key: KeyCode, is_start: bool) {
    match key {
        KeyCode::Esc => {
            // Apply and close
            if is_start {
                app.set_start_date_from_input();
            } else {
                app.set_end_date_from_input();
            }
            app.log_filter_panel = LogFilterPanel::None;
            app.log_selected = 0;
        }
        KeyCode::Char('s') => {
            // Switch to start date
            app.log_filter_panel = LogFilterPanel::StartDate;
        }
        KeyCode::Char('e') => {
            // Switch to end date
            app.log_filter_panel = LogFilterPanel::EndDate;
        }
        KeyCode::Enter => {
            // Apply the current date and close
            if is_start {
                app.set_start_date_from_input();
            } else {
                app.set_end_date_from_input();
            }
            app.log_filter_panel = LogFilterPanel::None;
            app.log_selected = 0;
        }
        KeyCode::Backspace => {
            if is_start {
                app.start_date_input.pop();
            } else {
                app.end_date_input.pop();
            }
        }
        KeyCode::Char(c) if c.is_ascii_digit() || c == '-' => {
            if is_start {
                if app.start_date_input.len() < 10 {
                    app.start_date_input.push(c);
                }
            } else {
                if app.end_date_input.len() < 10 {
                    app.end_date_input.push(c);
                }
            }
        }
        _ => {}
    }
}

fn handle_project_filter_input(app: &mut App, key: KeyCode) {
    let all_projects = app.all_project_names();

    match key {
        KeyCode::Esc => {
            app.log_filter_panel = LogFilterPanel::None;
            app.log_selected = 0;
        }
        KeyCode::Up => {
            if app.log_filter_project_selected > 0 {
                app.log_filter_project_selected -= 1;
            }
        }
        KeyCode::Down => {
            if app.log_filter_project_selected < all_projects.len().saturating_sub(1) {
                app.log_filter_project_selected += 1;
            }
        }
        KeyCode::Char('x') => {
            // Toggle the selected project
            if let Some(project) = all_projects.get(app.log_filter_project_selected) {
                app.toggle_log_filter_project(project);
            }
        }
        _ => {}
    }
}

fn handle_people_filter_input(app: &mut App, key: KeyCode) {
    let all_people = app.all_people_names();

    match key {
        KeyCode::Esc => {
            app.log_filter_panel = LogFilterPanel::None;
            app.log_selected = 0;
        }
        KeyCode::Up => {
            if app.log_filter_people_selected > 0 {
                app.log_filter_people_selected -= 1;
            }
        }
        KeyCode::Down => {
            if app.log_filter_people_selected < all_people.len().saturating_sub(1) {
                app.log_filter_people_selected += 1;
            }
        }
        KeyCode::Char('x') => {
            // Toggle the selected person
            if let Some(person) = all_people.get(app.log_filter_people_selected) {
                app.toggle_log_filter_person(person);
            }
        }
        _ => {}
    }
}

fn handle_view_log_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => {
            app.go_back();
        }
        KeyCode::Up => {
            if app.view_log_scroll > 0 {
                app.view_log_scroll -= 1;
            }
        }
        KeyCode::Down => {
            app.view_log_scroll += 1;
        }
        KeyCode::PageUp => {
            app.view_log_scroll = app.view_log_scroll.saturating_sub(10);
        }
        KeyCode::PageDown => {
            app.view_log_scroll += 10;
        }
        _ => {}
    }
}

fn handle_project_list_input(app: &mut App, key: KeyCode) -> Result<()> {
    // Handle filter panel input if one is open
    if app.project_filter_panel == ui::app::ProjectFilterPanel::Groups {
        handle_project_group_filter_input(app, key);
        return Ok(());
    }

    // Normal project list navigation
    match key {
        KeyCode::Esc => {
            app.go_to_screen(ui::app::Screen::Menu);
        }
        KeyCode::Up => {
            if app.project_selected > 0 {
                app.project_selected -= 1;
            }
        }
        KeyCode::Down => {
            if app.project_selected < app.filtered_projects.len().saturating_sub(1) {
                app.project_selected += 1;
            }
        }
        KeyCode::Enter => {
            app.show_project_details()?;
        }
        KeyCode::Char('g') => {
            // Open groups filter
            app.project_filter_group_selected = 0;
            app.project_filter_panel = ui::app::ProjectFilterPanel::Groups;
        }
        _ => {}
    }

    Ok(())
}

fn handle_project_group_filter_input(app: &mut App, key: KeyCode) {
    let all_groups = app.all_group_names();

    match key {
        KeyCode::Esc => {
            app.project_filter_panel = ui::app::ProjectFilterPanel::None;
            app.project_selected = 0;
        }
        KeyCode::Up => {
            if app.project_filter_group_selected > 0 {
                app.project_filter_group_selected -= 1;
            }
        }
        KeyCode::Down => {
            if app.project_filter_group_selected < all_groups.len().saturating_sub(1) {
                app.project_filter_group_selected += 1;
            }
        }
        KeyCode::Char('x') => {
            // Toggle the selected group
            if let Some(group) = all_groups.get(app.project_filter_group_selected) {
                app.toggle_project_filter_group(group);
            }
            app.project_selected = 0;
        }
        _ => {}
    }
}

fn handle_project_details_input(app: &mut App, key: KeyCode) -> Result<()> {
    match key {
        KeyCode::Esc => {
            app.go_back();
        }
        KeyCode::Up => {
            if app.project_details_log_selected > 0 {
                app.project_details_log_selected -= 1;
            }
        }
        KeyCode::Down => {
            if app.project_details_log_selected < app.project_details_logs.len().saturating_sub(1) {
                app.project_details_log_selected += 1;
            }
        }
        KeyCode::Enter => {
            app.view_project_details_log();
        }
        KeyCode::Char('e') => {
            app.start_edit_project_from_details();
        }
        _ => {}
    }

    Ok(())
}

fn handle_project_edit_input(app: &mut App, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
    // Check for Ctrl combinations
    if modifiers.contains(KeyModifiers::CONTROL) {
        match key {
            KeyCode::Char('s') => {
                app.save_edited_project()?;
                return Ok(());
            }
            _ => {}
        }
    }

    match key {
        KeyCode::Esc => {
            app.go_back();
        }
        KeyCode::Tab => {
            // Close any open dropdowns
            app.project_edit_status_dropdown_open = false;
            app.project_edit_group_dropdown_open = false;
            // Cycle to next field (0-4)
            app.project_edit_field = (app.project_edit_field + 1) % 5;
            // Open dropdown if moving to status or group field
            if app.project_edit_field == 3 {
                app.project_edit_status_dropdown_open = true;
            } else if app.project_edit_field == 4 {
                app.project_edit_group_dropdown_open = true;
            }
        }
        KeyCode::Char('1') => {
            app.project_edit_status_dropdown_open = false;
            app.project_edit_group_dropdown_open = false;
            app.project_edit_field = 0;
        }
        KeyCode::Char('2') => {
            app.project_edit_status_dropdown_open = false;
            app.project_edit_group_dropdown_open = false;
            app.project_edit_field = 1;
        }
        KeyCode::Char('3') => {
            app.project_edit_status_dropdown_open = false;
            app.project_edit_group_dropdown_open = false;
            app.project_edit_field = 2;
        }
        KeyCode::Char('4') => {
            app.project_edit_status_dropdown_open = false;
            app.project_edit_group_dropdown_open = false;
            app.project_edit_field = 3;
            app.project_edit_status_dropdown_open = true;
        }
        KeyCode::Char('5') => {
            app.project_edit_status_dropdown_open = false;
            app.project_edit_group_dropdown_open = false;
            app.project_edit_field = 4;
            app.project_edit_group_dropdown_open = true;
        }
        KeyCode::Up => {
            // Navigate dropdown if on status or group field
            if app.project_edit_field == 3 {
                app.project_edit_status_dropdown_open = true;
                if app.project_edit_status_dropdown_selected > 0 {
                    app.project_edit_status_dropdown_selected -= 1;
                }
            } else if app.project_edit_field == 4 {
                app.project_edit_group_dropdown_open = true;
                if app.project_edit_group_dropdown_selected > 0 {
                    app.project_edit_group_dropdown_selected -= 1;
                }
            }
        }
        KeyCode::Down => {
            // Navigate dropdown if on status or group field
            if app.project_edit_field == 3 {
                app.project_edit_status_dropdown_open = true;
                let max = app.config.allowed_state_names().len().saturating_sub(1);
                if app.project_edit_status_dropdown_selected < max {
                    app.project_edit_status_dropdown_selected += 1;
                }
            } else if app.project_edit_field == 4 {
                app.project_edit_group_dropdown_open = true;
                let max = app.config.allowed_groups().len(); // +1 for "(no group)" option, -1 for index
                if app.project_edit_group_dropdown_selected < max {
                    app.project_edit_group_dropdown_selected += 1;
                }
            }
        }
        KeyCode::Enter => {
            // Confirm dropdown selection
            if app.project_edit_field == 3 && app.project_edit_status_dropdown_open {
                let states = app.config.allowed_state_names();
                if let Some(selected) = states.get(app.project_edit_status_dropdown_selected) {
                    app.project_edit_status = selected.clone();
                }
                app.project_edit_status_dropdown_open = false;
            } else if app.project_edit_field == 4 && app.project_edit_group_dropdown_open {
                let mut groups = app.config.allowed_groups();
                groups.insert(0, "(no group)".to_string());
                if let Some(selected) = groups.get(app.project_edit_group_dropdown_selected) {
                    app.project_edit_group = if selected == "(no group)" {
                        String::new()
                    } else {
                        selected.clone()
                    };
                }
                app.project_edit_group_dropdown_open = false;
            }
        }
        KeyCode::Char(c) => {
            // Insert character into current field (only for text fields, not dropdowns)
            match app.project_edit_field {
                0 => app.project_edit_name.push(c),
                1 => app.project_edit_description.push(c),
                2 => app.project_edit_jira.push(c),
                // Fields 3 and 4 are dropdowns, no character input
                _ => {}
            }
        }
        KeyCode::Backspace => {
            // Delete character from current field (only for text fields, not dropdowns)
            match app.project_edit_field {
                0 => { app.project_edit_name.pop(); }
                1 => { app.project_edit_description.pop(); }
                2 => { app.project_edit_jira.pop(); }
                // Fields 3 and 4 are dropdowns, no backspace
                _ => {}
            }
        }
        _ => {}
    }

    Ok(())
}
