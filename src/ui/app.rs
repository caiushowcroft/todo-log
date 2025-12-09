use crate::models::{Config, LogEntry, LogFilter, Person, Project, Todo, TodoFilter};
use crate::storage::Storage;
use anyhow::Result;
use std::path::PathBuf;

/// The current screen/view of the application
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Menu,
    LogEntry,
    TodoList,
    LogList,
    ViewLog(PathBuf),
    ProjectList,
    ProjectDetails(usize), // Index in projects list
    ProjectEdit(Option<usize>), // None = new project, Some(idx) = edit existing
}

/// Main application state
pub struct App {
    pub storage: Storage,
    pub screen: Screen,
    pub previous_screen: Option<Screen>,
    pub running: bool,
    pub menu_selected: usize,

    // Data
    pub config: Config,
    pub projects: Vec<Project>,
    pub people: Vec<Person>,
    pub todos: Vec<Todo>,
    pub logs: Vec<LogEntry>,

    // Log entry editing state
    pub current_log: LogEntry,
    pub log_cursor_pos: usize,
    pub attachments: Vec<PathBuf>,
    pub autocomplete_suggestions: Vec<String>,
    pub autocomplete_index: usize,
    pub autocomplete_active: bool,
    pub autocomplete_type: AutocompleteType,

    // File browser state
    pub file_browser_open: bool,
    pub file_browser_dir: PathBuf,
    pub file_browser_entries: Vec<FileEntry>,
    pub file_browser_selected: usize,

    // Todo list state
    pub todo_filter: TodoFilter,
    pub todo_selected: usize,
    pub filtered_todos: Vec<Todo>,

    // Log list state
    pub log_filter: LogFilter,
    pub log_selected: usize,
    pub filtered_logs: Vec<LogEntry>,

    // View log state
    pub view_log_scroll: u16,

    // Todo filter editing state
    pub todo_filter_panel: TodoFilterPanel,
    pub todo_filter_project_selected: usize,
    pub todo_filter_people_selected: usize,

    // Log filter editing state
    pub log_filter_panel: LogFilterPanel,
    pub start_date_input: String,
    pub end_date_input: String,
    pub log_filter_project_selected: usize,
    pub log_filter_people_selected: usize,

    // Project list state
    pub project_selected: usize,
    pub project_list_scroll: usize,
    pub filtered_projects: Vec<Project>,
    pub project_filter_groups: Vec<String>,
    pub project_filter_panel: ProjectFilterPanel,
    pub project_filter_group_selected: usize,

    // Project details state
    pub project_details_log_selected: usize,
    pub project_details_logs: Vec<LogEntry>,

    // Project edit state
    pub editing_project: Option<Project>,
    pub project_edit_field: usize, // 0=name, 1=description, 2=jira, 3=status, 4=group
    pub project_edit_name: String,
    pub project_edit_description: String,
    pub project_edit_jira: String,
    pub project_edit_status: String,
    pub project_edit_group: String,
    pub project_edit_status_dropdown_open: bool,
    pub project_edit_status_dropdown_selected: usize,
    pub project_edit_group_dropdown_open: bool,
    pub project_edit_group_dropdown_selected: usize,

    // Status message
    pub status_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AutocompleteType {
    None,
    Project,
    Person,
}

/// Which filter panel is currently being edited in log list view
#[derive(Debug, Clone, PartialEq)]
pub enum LogFilterPanel {
    None,
    StartDate,
    EndDate,
    Projects,
    People,
}

/// Which filter panel is currently being edited in todo list view
#[derive(Debug, Clone, PartialEq)]
pub enum TodoFilterPanel {
    None,
    Completed,
    Projects,
    People,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectFilterPanel {
    None,
    Groups,
}

/// A file or directory entry in the file browser
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
}

impl App {
    pub fn new() -> Result<Self> {
        let storage = Storage::new()?;
        storage.initialize()?;

        let config = storage.load_config().unwrap_or_default();
        let projects = storage.load_projects().unwrap_or_default();
        let people = storage.load_people().unwrap_or_default();

        Ok(Self {
            storage,
            screen: Screen::Menu,
            previous_screen: None,
            running: true,
            menu_selected: 0,

            config,
            projects,
            people,
            todos: Vec::new(),
            logs: Vec::new(),

            current_log: LogEntry::new(),
            log_cursor_pos: 0,
            attachments: Vec::new(),
            autocomplete_suggestions: Vec::new(),
            autocomplete_index: 0,
            autocomplete_active: false,
            autocomplete_type: AutocompleteType::None,

            file_browser_open: false,
            file_browser_dir: dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
            file_browser_entries: Vec::new(),
            file_browser_selected: 0,

            todo_filter: TodoFilter::default(),
            todo_selected: 0,
            filtered_todos: Vec::new(),

            log_filter: LogFilter::default(),
            log_selected: 0,
            filtered_logs: Vec::new(),

            view_log_scroll: 0,

            todo_filter_panel: TodoFilterPanel::None,
            todo_filter_project_selected: 0,
            todo_filter_people_selected: 0,

            log_filter_panel: LogFilterPanel::None,
            start_date_input: String::new(),
            end_date_input: String::new(),
            log_filter_project_selected: 0,
            log_filter_people_selected: 0,

            project_selected: 0,
            project_list_scroll: 0,
            filtered_projects: Vec::new(),
            project_filter_groups: Vec::new(),
            project_filter_panel: ProjectFilterPanel::None,
            project_filter_group_selected: 0,

            project_details_log_selected: 0,
            project_details_logs: Vec::new(),

            editing_project: None,
            project_edit_field: 0,
            project_edit_name: String::new(),
            project_edit_description: String::new(),
            project_edit_jira: String::new(),
            project_edit_status: String::new(),
            project_edit_group: String::new(),
            project_edit_status_dropdown_open: false,
            project_edit_status_dropdown_selected: 0,
            project_edit_group_dropdown_open: false,
            project_edit_group_dropdown_selected: 0,

            status_message: None,
        })
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn go_to_screen(&mut self, screen: Screen) {
        self.previous_screen = Some(self.screen.clone());
        self.screen = screen.clone();

        // Reset menu selection when going to menu
        if matches!(screen, Screen::Menu) {
            self.menu_selected = 0;
        }
    }

    pub fn go_back(&mut self) {
        if let Some(prev) = self.previous_screen.take() {
            self.screen = prev;

            // Reset menu selection when going back to menu
            if matches!(self.screen, Screen::Menu) {
                self.menu_selected = 0;
            }
        } else {
            self.screen = Screen::Menu;
            self.menu_selected = 0;
        }
    }

    /// Start creating a new log entry
    pub fn start_new_log(&mut self) {
        self.current_log = LogEntry::new();
        self.log_cursor_pos = 0;
        self.attachments.clear();
        self.autocomplete_suggestions.clear();
        self.autocomplete_active = false;
        self.go_to_screen(Screen::LogEntry);
    }

    /// Save the current log entry
    pub fn save_log(&mut self) -> Result<()> {
        if self.current_log.content.trim().is_empty() {
            self.status_message = Some("Cannot save empty log entry".to_string());
            return Ok(());
        }

        let path = self.storage.save_log_entry(&self.current_log, &self.attachments)?;
        self.status_message = Some(format!("Log saved to {:?}", path));
        self.go_to_screen(Screen::Menu);
        Ok(())
    }

    /// Load todos and go to todo list screen
    pub fn show_todos(&mut self) -> Result<()> {
        self.todos = self.storage.load_all_todos()?;
        self.apply_todo_filter();
        self.todo_selected = 0;
        self.go_to_screen(Screen::TodoList);
        Ok(())
    }

    /// Apply the current todo filter
    pub fn apply_todo_filter(&mut self) {
        self.filtered_todos = self
            .todos
            .iter()
            .filter(|t| self.todo_filter.matches(t))
            .cloned()
            .collect();
    }

    /// Toggle the selected todo's completion status
    pub fn toggle_selected_todo(&mut self) -> Result<()> {
        if let Some(todo) = self.filtered_todos.get_mut(self.todo_selected) {
            todo.toggle()?;
            // Also update in the main list
            for t in &mut self.todos {
                if t.log_path == todo.log_path && t.line_number == todo.line_number {
                    t.completed = todo.completed;
                    break;
                }
            }
        }
        Ok(())
    }

    /// Load logs and go to log list screen
    pub fn show_logs(&mut self) -> Result<()> {
        self.logs = self.storage.load_all_logs()?;
        self.apply_log_filter();
        self.log_selected = 0;
        self.go_to_screen(Screen::LogList);
        Ok(())
    }

    /// Apply the current log filter
    pub fn apply_log_filter(&mut self) {
        self.filtered_logs = self
            .logs
            .iter()
            .filter(|l| self.log_filter.matches(l))
            .cloned()
            .collect();
    }

    /// View the selected log entry
    pub fn view_selected_log(&mut self) {
        if let Some(log) = self.filtered_logs.get(self.log_selected) {
            let path = log.file_path.clone();
            self.view_log_scroll = 0;
            self.go_to_screen(Screen::ViewLog(path));
        }
    }

    /// View log for the selected todo
    pub fn view_todo_log(&mut self) {
        if let Some(todo) = self.filtered_todos.get(self.todo_selected) {
            let path = todo.log_path.clone();
            self.view_log_scroll = 0;
            self.go_to_screen(Screen::ViewLog(path));
        }
    }

    /// View log from project details screen
    pub fn view_project_details_log(&mut self) {
        if let Some(log) = self.project_details_logs.get(self.project_details_log_selected) {
            let path = log.file_path.clone();
            self.view_log_scroll = 0;
            self.go_to_screen(Screen::ViewLog(path));
        }
    }

    /// Show projects and go to project list screen
    pub fn show_projects(&mut self) -> Result<()> {
        // Reload projects from file to ensure we have the latest data
        self.projects = self.storage.load_projects()?;
        self.apply_project_filter();
        self.project_selected = 0;
        self.project_list_scroll = 0;
        self.go_to_screen(Screen::ProjectList);
        Ok(())
    }

    /// Apply the current project filter
    pub fn apply_project_filter(&mut self) {
        self.filtered_projects = self.projects
            .iter()
            .filter(|p| {
                if self.project_filter_groups.is_empty() {
                    true
                } else {
                    self.project_filter_groups.contains(&p.group)
                }
            })
            .cloned()
            .collect();

        // Sort filtered projects to match the visual display order (grouped and sorted)
        self.sort_filtered_projects_by_display_order();
    }

    /// Sort filtered projects to match the display order (grouped, then sorted by group)
    fn sort_filtered_projects_by_display_order(&mut self) {
        self.filtered_projects.sort_by(|a, b| {
            let group_a = if a.group.is_empty() { "(No group)" } else { a.group.as_str() };
            let group_b = if b.group.is_empty() { "(No group)" } else { b.group.as_str() };

            // Sort groups alphabetically, with "(No group)" last
            let group_order = if group_a == "(No group)" && group_b != "(No group)" {
                std::cmp::Ordering::Greater
            } else if group_b == "(No group)" && group_a != "(No group)" {
                std::cmp::Ordering::Less
            } else {
                group_a.cmp(group_b)
            };

            // If in the same group, maintain original order
            if group_order == std::cmp::Ordering::Equal {
                std::cmp::Ordering::Equal
            } else {
                group_order
            }
        });
    }

    /// Get all unique group names from projects
    pub fn all_group_names(&self) -> Vec<String> {
        let mut groups: Vec<String> = self.projects
            .iter()
            .map(|p| p.group.clone())
            .collect();
        groups.sort();
        groups.dedup();
        // Move empty string to the end if it exists
        if let Some(pos) = groups.iter().position(|g| g.is_empty()) {
            groups.remove(pos);
            groups.push(String::from("(No group)"));
        }
        groups
    }

    /// Toggle a group in the project filter
    pub fn toggle_project_filter_group(&mut self, group: &str) {
        // Convert "(No group)" back to empty string
        let group = if group == "(No group)" { "" } else { group };

        if let Some(pos) = self.project_filter_groups.iter().position(|g| g == group) {
            self.project_filter_groups.remove(pos);
        } else {
            self.project_filter_groups.push(group.to_string());
        }
        self.apply_project_filter();
    }

    /// Show project details for the selected project
    pub fn show_project_details(&mut self) -> Result<()> {
        if let Some(project) = self.filtered_projects.get(self.project_selected).cloned() {
            // Find the index in the original projects list
            if let Some(idx) = self.projects.iter().position(|p| p.name == project.name) {
                // Load all logs
                self.logs = self.storage.load_all_logs()?;

                // Filter logs that contain this project
                self.project_details_logs = self.logs
                    .iter()
                    .filter(|log| log.projects.contains(&project.name))
                    .cloned()
                    .collect();

                self.project_details_log_selected = 0;
                self.go_to_screen(Screen::ProjectDetails(idx));
            }
        }
        Ok(())
    }

    /// Start editing a project from the details screen
    pub fn start_edit_project_from_details(&mut self) {
        if let Screen::ProjectDetails(idx) = self.screen {
            if let Some(project) = self.projects.get(idx) {
                self.project_edit_name = project.name.clone();
                self.project_edit_description = project.description.clone().unwrap_or_default();
                self.project_edit_jira = project.jira.clone().unwrap_or_default();
                self.project_edit_status = project.status.clone();
                self.project_edit_group = project.group.clone();
                self.project_edit_field = 0;

                // Initialize dropdown selected indices
                let states = self.config.allowed_state_names();
                self.project_edit_status_dropdown_selected = states
                    .iter()
                    .position(|s| s == &project.status)
                    .unwrap_or(0);

                let mut groups = self.config.allowed_groups();
                groups.insert(0, "(no group)".to_string());
                let group_to_find = if project.group.is_empty() {
                    "(no group)"
                } else {
                    &project.group
                };
                self.project_edit_group_dropdown_selected = groups
                    .iter()
                    .position(|g| g == group_to_find)
                    .unwrap_or(0);

                self.project_edit_status_dropdown_open = false;
                self.project_edit_group_dropdown_open = false;

                self.go_to_screen(Screen::ProjectEdit(Some(idx)));
            }
        }
    }

    /// Start editing the selected project
    pub fn start_edit_project(&mut self) {
        if let Some(project) = self.filtered_projects.get(self.project_selected).cloned() {
            self.project_edit_name = project.name.clone();
            self.project_edit_description = project.description.clone().unwrap_or_default();
            self.project_edit_jira = project.jira.clone().unwrap_or_default();
            self.project_edit_status = project.status.clone();
            self.project_edit_group = project.group.clone();
            self.project_edit_field = 0;

            // Initialize dropdown selected indices
            let states = self.config.allowed_state_names();
            self.project_edit_status_dropdown_selected = states
                .iter()
                .position(|s| s == &project.status)
                .unwrap_or(0);

            let mut groups = self.config.allowed_groups();
            groups.insert(0, "(no group)".to_string());
            let group_to_find = if project.group.is_empty() {
                "(no group)"
            } else {
                &project.group
            };
            self.project_edit_group_dropdown_selected = groups
                .iter()
                .position(|g| g == group_to_find)
                .unwrap_or(0);

            self.project_edit_status_dropdown_open = false;
            self.project_edit_group_dropdown_open = false;

            // Find the index in the original projects list
            if let Some(idx) = self.projects.iter().position(|p| p.name == project.name) {
                self.go_to_screen(Screen::ProjectEdit(Some(idx)));
            }
        }
    }

    /// Start creating a new project
    pub fn start_new_project(&mut self) {
        // Initialize empty fields
        self.project_edit_name = String::new();
        self.project_edit_description = String::new();
        self.project_edit_jira = String::new();

        // Initialize with first available status
        let states = self.config.allowed_state_names();
        self.project_edit_status = states.first().cloned().unwrap_or_else(|| "open".to_string());
        self.project_edit_status_dropdown_selected = 0;

        // Initialize with no group
        self.project_edit_group = String::new();
        self.project_edit_group_dropdown_selected = 0; // "(no group)" is at index 0

        self.project_edit_field = 0;
        self.project_edit_status_dropdown_open = false;
        self.project_edit_group_dropdown_open = false;

        self.go_to_screen(Screen::ProjectEdit(None));
    }

    /// Save the edited project
    pub fn save_edited_project(&mut self) -> Result<()> {
        if let Screen::ProjectEdit(idx_opt) = self.screen {
            match idx_opt {
                Some(idx) => {
                    // Editing existing project
                    if let Some(project) = self.projects.get_mut(idx) {
                        project.name = self.project_edit_name.clone();
                        project.description = if self.project_edit_description.is_empty() {
                            None
                        } else {
                            Some(self.project_edit_description.clone())
                        };
                        project.jira = if self.project_edit_jira.is_empty() {
                            None
                        } else {
                            Some(self.project_edit_jira.clone())
                        };
                        project.status = self.project_edit_status.clone();
                        project.group = self.project_edit_group.clone();

                        self.storage.save_projects(&self.projects)?;
                        self.status_message = Some("Project saved".to_string());
                    }
                }
                None => {
                    // Creating new project
                    let new_project = Project {
                        name: self.project_edit_name.clone(),
                        description: if self.project_edit_description.is_empty() {
                            None
                        } else {
                            Some(self.project_edit_description.clone())
                        },
                        jira: if self.project_edit_jira.is_empty() {
                            None
                        } else {
                            Some(self.project_edit_jira.clone())
                        },
                        status: self.project_edit_status.clone(),
                        group: self.project_edit_group.clone(),
                    };

                    self.projects.push(new_project);
                    self.storage.save_projects(&self.projects)?;
                    self.status_message = Some("Project created".to_string());
                }
            }

            // Reload projects from file to ensure consistency
            self.projects = self.storage.load_projects()?;

            // Reapply the project filter to update the view
            self.apply_project_filter();

            self.go_back();
        }
        Ok(())
    }

    /// Update autocomplete suggestions based on current input
    pub fn update_autocomplete(&mut self) {
        let content = &self.current_log.content;
        if self.log_cursor_pos == 0 || content.is_empty() {
            self.autocomplete_active = false;
            return;
        }

        // Get the text before cursor
        let before_cursor: String = content.chars().take(self.log_cursor_pos).collect();

        // Find the last word being typed
        let last_word_start = before_cursor
            .rfind(|c: char| c.is_whitespace())
            .map(|i| i + 1)
            .unwrap_or(0);

        let current_word: String = before_cursor.chars().skip(last_word_start).collect();

        if current_word.starts_with('#') && current_word.len() > 1 {
            // Project autocomplete
            let prefix = &current_word[1..].to_lowercase();
            self.autocomplete_suggestions = self
                .projects
                .iter()
                .filter(|p| p.name.to_lowercase().starts_with(prefix))
                .map(|p| p.name.clone())
                .collect();
            self.autocomplete_type = AutocompleteType::Project;
            self.autocomplete_active = !self.autocomplete_suggestions.is_empty();
            self.autocomplete_index = 0;
        } else if current_word.starts_with('@') && current_word.len() > 1 {
            // Person autocomplete
            let prefix = &current_word[1..].to_lowercase();
            self.autocomplete_suggestions = self
                .people
                .iter()
                .filter(|p| p.name.to_lowercase().starts_with(prefix))
                .map(|p| p.name.clone())
                .collect();
            self.autocomplete_type = AutocompleteType::Person;
            self.autocomplete_active = !self.autocomplete_suggestions.is_empty();
            self.autocomplete_index = 0;
        } else {
            self.autocomplete_active = false;
            self.autocomplete_type = AutocompleteType::None;
        }
    }

    /// Accept the current autocomplete suggestion
    pub fn accept_autocomplete(&mut self) {
        if !self.autocomplete_active || self.autocomplete_suggestions.is_empty() {
            return;
        }

        let suggestion = self.autocomplete_suggestions[self.autocomplete_index].clone();
        let content = &self.current_log.content;
        let before_cursor: String = content.chars().take(self.log_cursor_pos).collect();

        // Find the start of the current tag
        let last_word_start = before_cursor
            .rfind(|c: char| c.is_whitespace())
            .map(|i| i + 1)
            .unwrap_or(0);

        // Get the prefix character (# or @)
        let prefix = match self.autocomplete_type {
            AutocompleteType::Project => "#",
            AutocompleteType::Person => "@",
            AutocompleteType::None => return,
        };

        // Build new content
        let before: String = content.chars().take(last_word_start).collect();
        let after: String = content.chars().skip(self.log_cursor_pos).collect();
        let new_word = format!("{}{} ", prefix, suggestion);

        self.current_log.content = format!("{}{}{}", before, new_word, after);
        self.log_cursor_pos = last_word_start + new_word.len();

        self.autocomplete_active = false;
        self.autocomplete_suggestions.clear();
    }

    /// Insert a character at the current cursor position
    pub fn insert_char(&mut self, c: char) {
        let before: String = self.current_log.content.chars().take(self.log_cursor_pos).collect();
        let after: String = self.current_log.content.chars().skip(self.log_cursor_pos).collect();
        self.current_log.content = format!("{}{}{}", before, c, after);
        self.log_cursor_pos += 1;
        self.update_autocomplete();
    }

    /// Delete character before cursor
    pub fn delete_char(&mut self) {
        if self.log_cursor_pos > 0 {
            let before: String = self.current_log.content.chars().take(self.log_cursor_pos - 1).collect();
            let after: String = self.current_log.content.chars().skip(self.log_cursor_pos).collect();
            self.current_log.content = format!("{}{}", before, after);
            self.log_cursor_pos -= 1;
            self.update_autocomplete();
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.log_cursor_pos > 0 {
            self.log_cursor_pos -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.log_cursor_pos < self.current_log.content.chars().count() {
            self.log_cursor_pos += 1;
        }
    }

    /// Move cursor up one line
    pub fn move_cursor_up(&mut self) {
        let content = &self.current_log.content;
        let before_cursor: String = content.chars().take(self.log_cursor_pos).collect();
        let lines: Vec<&str> = before_cursor.split('\n').collect();

        if lines.len() <= 1 {
            // Already on first line
            return;
        }

        let current_line_idx = lines.len() - 1;
        let current_col = lines[current_line_idx].chars().count();

        // Get previous line
        let prev_line = lines[current_line_idx - 1];
        let prev_line_len = prev_line.chars().count();

        // Move to same column on previous line (or end of line if shorter)
        let target_col = current_col.min(prev_line_len);

        // Calculate new cursor position
        let chars_before_prev_line: usize = lines[..current_line_idx - 1]
            .iter()
            .map(|l| l.chars().count() + 1) // +1 for newline
            .sum();

        self.log_cursor_pos = chars_before_prev_line + target_col;
    }

    /// Move cursor down one line
    pub fn move_cursor_down(&mut self) {
        let content = &self.current_log.content;
        let all_lines: Vec<&str> = content.split('\n').collect();

        // Find current line and column
        let before_cursor: String = content.chars().take(self.log_cursor_pos).collect();
        let lines_before: Vec<&str> = before_cursor.split('\n').collect();
        let current_line_idx = lines_before.len() - 1;
        let current_col = lines_before[current_line_idx].chars().count();

        if current_line_idx >= all_lines.len() - 1 {
            // Already on last line
            return;
        }

        // Get next line
        let next_line = all_lines[current_line_idx + 1];
        let next_line_len = next_line.chars().count();

        // Move to same column on next line (or end of line if shorter)
        let target_col = current_col.min(next_line_len);

        // Calculate new cursor position
        let chars_before_next_line: usize = all_lines[..=current_line_idx]
            .iter()
            .map(|l| l.chars().count() + 1) // +1 for newline
            .sum();

        self.log_cursor_pos = chars_before_next_line + target_col;
    }

    /// Add an attachment
    pub fn add_attachment(&mut self, path: PathBuf) {
        if path.exists() {
            self.attachments.push(path);
            self.status_message = Some("Attachment added".to_string());
        } else {
            self.status_message = Some("File not found".to_string());
        }
    }

    /// Get all unique project names from loaded data
    pub fn all_project_names(&self) -> Vec<String> {
        self.projects.iter().map(|p| p.name.clone()).collect()
    }

    /// Get all unique people names from loaded data
    pub fn all_people_names(&self) -> Vec<String> {
        self.people.iter().map(|p| p.name.clone()).collect()
    }

    /// Clear status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Toggle a project in the log filter
    pub fn toggle_log_filter_project(&mut self, project: &str) {
        if self.log_filter.projects.contains(&project.to_string()) {
            self.log_filter.projects.retain(|p| p != project);
        } else {
            self.log_filter.projects.push(project.to_string());
        }
        self.apply_log_filter();
    }

    /// Toggle a person in the log filter
    pub fn toggle_log_filter_person(&mut self, person: &str) {
        if self.log_filter.people.contains(&person.to_string()) {
            self.log_filter.people.retain(|p| p != person);
        } else {
            self.log_filter.people.push(person.to_string());
        }
        self.apply_log_filter();
    }

    /// Parse and set the start date from input string
    pub fn set_start_date_from_input(&mut self) {
        use chrono::NaiveDate;
        if self.start_date_input.is_empty() {
            self.log_filter.start_date = None;
        } else if let Ok(date) = NaiveDate::parse_from_str(&self.start_date_input, "%Y-%m-%d") {
            self.log_filter.start_date = Some(date);
        }
        self.apply_log_filter();
    }

    /// Parse and set the end date from input string
    pub fn set_end_date_from_input(&mut self) {
        use chrono::NaiveDate;
        if self.end_date_input.is_empty() {
            self.log_filter.end_date = None;
        } else if let Ok(date) = NaiveDate::parse_from_str(&self.end_date_input, "%Y-%m-%d") {
            self.log_filter.end_date = Some(date);
        }
        self.apply_log_filter();
    }

    /// Initialize date input fields from current filter values
    pub fn init_date_inputs(&mut self) {
        self.start_date_input = self
            .log_filter
            .start_date
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default();
        self.end_date_input = self
            .log_filter
            .end_date
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default();
    }

    /// Toggle a project in the todo filter
    pub fn toggle_todo_filter_project(&mut self, project: &str) {
        if self.todo_filter.projects.contains(&project.to_string()) {
            self.todo_filter.projects.retain(|p| p != project);
        } else {
            self.todo_filter.projects.push(project.to_string());
        }
        self.apply_todo_filter();
    }

    /// Toggle a person in the todo filter
    pub fn toggle_todo_filter_person(&mut self, person: &str) {
        if self.todo_filter.people.contains(&person.to_string()) {
            self.todo_filter.people.retain(|p| p != person);
        } else {
            self.todo_filter.people.push(person.to_string());
        }
        self.apply_todo_filter();
    }

    /// Open the file browser
    pub fn open_file_browser(&mut self) {
        self.file_browser_open = true;
        self.file_browser_selected = 0;
        self.load_directory_contents();
    }

    /// Close the file browser
    pub fn close_file_browser(&mut self) {
        self.file_browser_open = false;
    }

    /// Load the contents of the current directory
    pub fn load_directory_contents(&mut self) {
        use std::fs;

        self.file_browser_entries.clear();

        // Add parent directory entry if not at root
        if let Some(parent) = self.file_browser_dir.parent() {
            self.file_browser_entries.push(FileEntry {
                name: "..".to_string(),
                path: parent.to_path_buf(),
                is_dir: true,
                size: 0,
            });
        }

        // Read directory contents
        if let Ok(entries) = fs::read_dir(&self.file_browser_dir) {
            let mut dirs: Vec<FileEntry> = Vec::new();
            let mut files: Vec<FileEntry> = Vec::new();

            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                // Skip hidden files
                if name.starts_with('.') {
                    continue;
                }

                let metadata = entry.metadata().ok();
                let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);

                let entry = FileEntry {
                    name,
                    path,
                    is_dir,
                    size,
                };

                if is_dir {
                    dirs.push(entry);
                } else {
                    files.push(entry);
                }
            }

            // Sort directories and files alphabetically
            dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            // Add directories first, then files
            self.file_browser_entries.extend(dirs);
            self.file_browser_entries.extend(files);
        }
    }

    /// Navigate into a directory or select a file
    pub fn file_browser_enter(&mut self) {
        if let Some(entry) = self.file_browser_entries.get(self.file_browser_selected).cloned() {
            if entry.is_dir {
                // Navigate into directory
                self.file_browser_dir = entry.path;
                self.file_browser_selected = 0;
                self.load_directory_contents();
            } else {
                // Select file as attachment
                if !self.attachments.contains(&entry.path) {
                    self.attachments.push(entry.path);
                    self.status_message = Some(format!("Added attachment: {}", entry.name));
                }
                self.close_file_browser();
            }
        }
    }

    /// Navigate to parent directory
    pub fn file_browser_go_up(&mut self) {
        if let Some(parent) = self.file_browser_dir.parent() {
            self.file_browser_dir = parent.to_path_buf();
            self.file_browser_selected = 0;
            self.load_directory_contents();
        }
    }

    /// Remove an attachment by index
    pub fn remove_attachment(&mut self, index: usize) {
        if index < self.attachments.len() {
            let removed = self.attachments.remove(index);
            if let Some(name) = removed.file_name() {
                self.status_message = Some(format!("Removed: {}", name.to_string_lossy()));
            }
        }
    }
}
