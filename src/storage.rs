use crate::models::{Config, LogEntry, Person, Project, Todo};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Storage manager for the todo-log application
pub struct Storage {
    pub base_dir: PathBuf,
}

impl Storage {
    pub fn new() -> Result<Self> {
        let base_dir = Self::default_base_dir()?;
        Ok(Self { base_dir })
    }

    #[allow(dead_code)] // Utility constructor for tests or custom paths
    pub fn with_base_dir(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    fn default_base_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        Ok(home.join("todo-log"))
    }

    pub fn projects_file(&self) -> PathBuf {
        self.base_dir.join("projects.yml")
    }

    pub fn people_file(&self) -> PathBuf {
        self.base_dir.join("people.yml")
    }

    pub fn config_file(&self) -> PathBuf {
        self.base_dir.join("config.yml")
    }

    /// Initialize the storage directory with example files if it doesn't exist
    pub fn initialize(&self) -> Result<()> {
        if !self.base_dir.exists() {
            fs::create_dir_all(&self.base_dir)
                .context("Failed to create todo-log directory")?;

            // Create example projects.yml
            let example_projects = vec![Project::example()];
            let projects_yaml = serde_yaml::to_string(&example_projects)?;
            fs::write(self.projects_file(), projects_yaml)
                .context("Failed to create projects.yml")?;

            // Create example people.yml
            let example_people = vec![Person::example()];
            let people_yaml = serde_yaml::to_string(&example_people)?;
            fs::write(self.people_file(), people_yaml)
                .context("Failed to create people.yml")?;

            // Create example config.yml
            let example_config = Config::default();
            let config_yaml = serde_yaml::to_string(&example_config)?;
            fs::write(self.config_file(), config_yaml)
                .context("Failed to create config.yml")?;
        }

        Ok(())
    }

    /// Load all projects from the projects.yml file
    pub fn load_projects(&self) -> Result<Vec<Project>> {
        let path = self.projects_file();
        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&path)
            .context("Failed to read projects.yml")?;

        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        let projects: Vec<Project> = serde_yaml::from_str(&content)
            .context("Failed to parse projects.yml")?;

        Ok(projects)
    }

    /// Save projects to the projects.yml file
    pub fn save_projects(&self, projects: &[Project]) -> Result<()> {
        let path = self.projects_file();
        let yaml = serde_yaml::to_string(projects)
            .context("Failed to serialize projects")?;
        fs::write(&path, yaml)
            .context("Failed to write projects.yml")?;
        Ok(())
    }

    /// Load all people from the people.yml file
    pub fn load_people(&self) -> Result<Vec<Person>> {
        let path = self.people_file();
        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&path)
            .context("Failed to read people.yml")?;

        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        let people: Vec<Person> = serde_yaml::from_str(&content)
            .context("Failed to parse people.yml")?;

        Ok(people)
    }

    /// Save people to the people.yml file
    pub fn save_people(&self, people: &[Person]) -> Result<()> {
        let path = self.people_file();
        let yaml = serde_yaml::to_string(people)
            .context("Failed to serialize people")?;
        fs::write(&path, yaml)
            .context("Failed to write people.yml")?;
        Ok(())
    }

    /// Load configuration from config.yml file
    pub fn load_config(&self) -> Result<Config> {
        let path = self.config_file();
        if !path.exists() {
            // Return default config if file doesn't exist
            return Ok(Config::default());
        }

        let content = fs::read_to_string(&path)
            .context("Failed to read config.yml")?;

        if content.trim().is_empty() {
            return Ok(Config::default());
        }

        let config: Config = serde_yaml::from_str(&content)
            .context("Failed to parse config.yml")?;

        Ok(config)
    }

    /// Save a log entry to disk
    pub fn save_log_entry(&self, entry: &LogEntry, attachments: &[PathBuf]) -> Result<PathBuf> {
        let year_dir = self.base_dir.join(format!("log-{}", entry.year()));
        let entry_dir = year_dir.join(entry.dir_name());

        fs::create_dir_all(&entry_dir)
            .context("Failed to create log entry directory")?;

        let log_file = entry_dir.join("log.txt");
        fs::write(&log_file, &entry.content)
            .context("Failed to write log file")?;

        // Copy attachments
        for attachment in attachments {
            if attachment.exists() {
                if let Some(filename) = attachment.file_name() {
                    let dest = entry_dir.join(filename);
                    fs::copy(attachment, &dest)
                        .context("Failed to copy attachment")?;
                }
            }
        }

        Ok(log_file)
    }

    /// Load all log entries from disk
    pub fn load_all_logs(&self) -> Result<Vec<LogEntry>> {
        let mut entries = Vec::new();

        // Find all log-* directories
        for entry in fs::read_dir(&self.base_dir).into_iter().flatten() {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with("log-") {
                            // This is a year directory, scan for log entries
                            for sub_entry in WalkDir::new(&path).min_depth(1).max_depth(1) {
                                if let Ok(sub_entry) = sub_entry {
                                    let log_file = sub_entry.path().join("log.txt");
                                    if log_file.exists() {
                                        if let Ok(content) = fs::read_to_string(&log_file) {
                                            let log_entry = LogEntry::parse(&content, log_file);
                                            entries.push(log_entry);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(entries)
    }

    /// Load all todos from all log entries
    pub fn load_all_todos(&self) -> Result<Vec<Todo>> {
        let entries = self.load_all_logs()?;
        let mut todos = Vec::new();

        for entry in entries {
            todos.extend(entry.todos);
        }

        Ok(todos)
    }

    /// Get a specific log entry by its file path
    #[allow(dead_code)] // Utility method for loading individual log entries
    pub fn load_log_by_path(&self, path: &PathBuf) -> Result<Option<LogEntry>> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            Ok(Some(LogEntry::parse(&content, path.clone())))
        } else {
            Ok(None)
        }
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new().expect("Failed to create default storage")
    }
}
