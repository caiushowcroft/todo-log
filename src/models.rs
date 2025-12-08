use chrono::{DateTime, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A project that can be tagged in log entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    #[serde(default)]
    pub jira: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub group: String,
}

fn default_status() -> String {
    "open".to_string()
}

impl Project {
    pub fn example() -> Self {
        Self {
            name: "new-website".to_string(),
            jira: Some("https://jira.com/projects/WWW-123".to_string()),
            description: Some("A project to create a new look on our website".to_string()),
            status: "open".to_string(),
            group: String::new(),
        }
    }
}

/// A person that can be tagged in log entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub name: String,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub tel: Option<String>,
    #[serde(default)]
    pub company: Option<String>,
}

impl Person {
    pub fn example() -> Self {
        Self {
            name: "john".to_string(),
            full_name: Some("John Smith".to_string()),
            email: Some("john@example.com".to_string()),
            tel: Some("555 123 3333".to_string()),
            company: Some("foo works".to_string()),
        }
    }
}

/// A log entry containing text, project/people tags, and todos
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub content: String,
    pub projects: Vec<String>,
    pub people: Vec<String>,
    pub todos: Vec<Todo>,
    pub attachments: Vec<PathBuf>,
    pub file_path: PathBuf,
}

impl LogEntry {
    pub fn new() -> Self {
        Self {
            timestamp: Local::now(),
            content: String::new(),
            projects: Vec::new(),
            people: Vec::new(),
            todos: Vec::new(),
            attachments: Vec::new(),
            file_path: PathBuf::new(),
        }
    }

    /// Parse a log entry from file content
    pub fn parse(content: &str, file_path: PathBuf) -> Self {
        let mut entry = Self::new();
        entry.content = content.to_string();
        entry.file_path = file_path;

        // Extract timestamp from path if possible
        if let Some(datetime_str) = entry.file_path.parent().and_then(|p| p.file_name()) {
            if let Some(dt_str) = datetime_str.to_str() {
                if let Ok(dt) = DateTime::parse_from_str(
                    &format!("{} +0000", dt_str),
                    "%Y-%m-%d_%H-%M-%S %z",
                ) {
                    entry.timestamp = dt.with_timezone(&Local);
                }
            }
        }

        // Extract projects (words starting with #)
        for word in content.split_whitespace() {
            if word.starts_with('#') && word.len() > 1 {
                let project = word[1..].trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_');
                if !project.is_empty() && !entry.projects.contains(&project.to_string()) {
                    entry.projects.push(project.to_string());
                }
            }
        }

        // Extract people (words starting with @)
        for word in content.split_whitespace() {
            if word.starts_with('@') && word.len() > 1 {
                let person = word[1..].trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_');
                if !person.is_empty() && !entry.people.contains(&person.to_string()) {
                    entry.people.push(person.to_string());
                }
            }
        }

        // Extract todos (lines starting with [] or [x])
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("[]") {
                let text = trimmed[2..].trim().to_string();
                entry.todos.push(Todo {
                    text,
                    completed: false,
                    line_number: line_num,
                    projects: entry.projects.clone(),
                    people: entry.people.clone(),
                    log_path: entry.file_path.clone(),
                });
            } else if trimmed.starts_with("[x]") || trimmed.starts_with("[X]") {
                let text = trimmed[3..].trim().to_string();
                entry.todos.push(Todo {
                    text,
                    completed: true,
                    line_number: line_num,
                    projects: entry.projects.clone(),
                    people: entry.people.clone(),
                    log_path: entry.file_path.clone(),
                });
            }
        }

        entry
    }

    /// Get the first line of the log entry for preview
    pub fn first_line(&self) -> &str {
        self.content.lines().next().unwrap_or("")
    }

    /// Get the directory name for this log entry
    pub fn dir_name(&self) -> String {
        self.timestamp.format("%Y-%m-%d_%H-%M-%S").to_string()
    }

    /// Get the year for directory organization
    pub fn year(&self) -> i32 {
        self.timestamp.format("%Y").to_string().parse().unwrap_or(2024)
    }
}

impl Default for LogEntry {
    fn default() -> Self {
        Self::new()
    }
}

/// A todo item extracted from a log entry
#[derive(Debug, Clone)]
pub struct Todo {
    pub text: String,
    pub completed: bool,
    pub line_number: usize,
    pub projects: Vec<String>,
    pub people: Vec<String>,
    pub log_path: PathBuf,
}

impl Todo {
    /// Toggle the completion status of this todo in the log file
    pub fn toggle(&mut self) -> anyhow::Result<()> {
        use std::fs;

        let content = fs::read_to_string(&self.log_path)?;
        let lines: Vec<&str> = content.lines().collect();

        let mut new_lines: Vec<String> = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            if i == self.line_number {
                let trimmed = line.trim();
                if self.completed {
                    // Change [x] to []
                    let new_line = line.replace("[x]", "[]").replace("[X]", "[]");
                    new_lines.push(new_line);
                } else {
                    // Change [] to [x]
                    let new_line = if trimmed.starts_with("[]") {
                        line.replacen("[]", "[x]", 1)
                    } else {
                        line.to_string()
                    };
                    new_lines.push(new_line);
                }
            } else {
                new_lines.push(line.to_string());
            }
        }

        fs::write(&self.log_path, new_lines.join("\n"))?;
        self.completed = !self.completed;
        Ok(())
    }
}

/// Filter configuration for todo list view
#[derive(Debug, Clone, Default)]
pub struct TodoFilter {
    pub show_completed: bool,
    pub projects: Vec<String>,
    pub people: Vec<String>,
}

impl TodoFilter {
    pub fn matches(&self, todo: &Todo) -> bool {
        // Filter by completion status
        if !self.show_completed && todo.completed {
            return false;
        }

        // Filter by projects (if any selected)
        if !self.projects.is_empty() {
            let has_matching_project = todo.projects.iter().any(|p| self.projects.contains(p));
            if !has_matching_project {
                return false;
            }
        }

        // Filter by people (if any selected)
        if !self.people.is_empty() {
            let has_matching_person = todo.people.iter().any(|p| self.people.contains(p));
            if !has_matching_person {
                return false;
            }
        }

        true
    }
}

/// Filter configuration for log list view
#[derive(Debug, Clone, Default)]
pub struct LogFilter {
    pub projects: Vec<String>,
    pub people: Vec<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

impl LogFilter {
    pub fn matches(&self, entry: &LogEntry) -> bool {
        // Filter by projects
        if !self.projects.is_empty() {
            let has_matching_project = entry.projects.iter().any(|p| self.projects.contains(p));
            if !has_matching_project {
                return false;
            }
        }

        // Filter by people
        if !self.people.is_empty() {
            let has_matching_person = entry.people.iter().any(|p| self.people.contains(p));
            if !has_matching_person {
                return false;
            }
        }

        // Filter by date range
        let entry_date = entry.timestamp.date_naive();
        if let Some(start) = self.start_date {
            if entry_date < start {
                return false;
            }
        }
        if let Some(end) = self.end_date {
            if entry_date > end {
                return false;
            }
        }

        true
    }
}
