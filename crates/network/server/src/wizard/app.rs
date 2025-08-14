use anyhow::anyhow;
use ratatui::Terminal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use time::OffsetDateTime;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Setup,
    Overview,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActivePanel {
    Categories,
    Settings,
    None,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SettingsCategory {
    Network,
    Database,
    Security,
    Storage,
    Logging,
    Performance,
    Features,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    All,
    Passwords,
    Environment,
    Notes,
}

impl ViewMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            ViewMode::All => "Items",
            ViewMode::Passwords => "Passwords",
            ViewMode::Environment => "Environment",
            ViewMode::Notes => "Notes",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatusType {
    Info,
    Success,
    Warning,
    Error,
}

impl FromStr for SettingsCategory {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result = match s.to_lowercase().as_str() {
            "network" => SettingsCategory::Network,
            "database" => SettingsCategory::Database,
            "security" => SettingsCategory::Security,
            "storage" => SettingsCategory::Storage,
            "logging" => SettingsCategory::Logging,
            "performance" => SettingsCategory::Performance,
            "features" => SettingsCategory::Features,
            _ => return Err(anyhow!("Invalid category: '{}'", s)),
        };
        Ok(result)
    }
}

impl SettingsCategory {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            SettingsCategory::Network => "network",
            SettingsCategory::Database => "database",
            SettingsCategory::Security => "security",
            SettingsCategory::Storage => "storage",
            SettingsCategory::Logging => "logging",
            SettingsCategory::Performance => "performance",
            SettingsCategory::Features => "features",
        }
    }

    #[must_use]
    pub const fn all() -> &'static [SettingsCategory] {
        &[
            SettingsCategory::Network,
            SettingsCategory::Database,
            SettingsCategory::Security,
            SettingsCategory::Storage,
            SettingsCategory::Logging,
            SettingsCategory::Performance,
            SettingsCategory::Features,
        ]
    }

    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            SettingsCategory::Network => "Network Configuration",
            SettingsCategory::Database => "Database Settings",
            SettingsCategory::Security => "Security & Authentication",
            SettingsCategory::Storage => "File Storage",
            SettingsCategory::Logging => "Logging & Monitoring",
            SettingsCategory::Performance => "Performance Tuning",
            SettingsCategory::Features => "Feature Flags",
        }
    }

    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            SettingsCategory::Network => "Configure server ports, SSL, and network protocols",
            SettingsCategory::Database => "Database connection strings and migration settings",
            SettingsCategory::Security => {
                "Authentication methods, encryption keys, and permissions"
            }
            SettingsCategory::Storage => {
                "File upload paths, storage backends, and cleanup policies"
            }
            SettingsCategory::Logging => "Log levels, output formats, and monitoring endpoints",
            SettingsCategory::Performance => "Cache settings, worker processes, and memory limits",
            SettingsCategory::Features => "Enable or disable experimental features and modules",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SettingsItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: SettingsCategory,
    pub subcategory: Option<String>,
    pub value: Option<String>,
    pub default_value: String,
    pub required: bool,
    pub completed: bool,
}

#[derive(Debug, Clone)]
pub struct SubcategoryProgress {
    pub name: String,
    pub completed_count: usize,
    pub total_count: usize,
    pub is_completed: bool,
}

#[derive(Debug, Clone)]
pub struct CategoryProgress {
    pub category: SettingsCategory,
    pub completed_count: usize,
    pub total_count: usize,
    pub is_completed: bool,
    pub subcategories: Vec<SubcategoryProgress>,
}

#[derive(Debug, Clone)]
pub enum CategoryListItem {
    Category(SettingsCategory, usize, usize, bool), // category, completed, total, is_completed
    Subcategory(SettingsCategory, String, usize, usize, bool), // parent_category, name, completed, total, is_completed
}

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub screen: Screen,
    pub active_panel: ActivePanel,

    pub categories: Vec<CategoryProgress>,
    pub category_list_items: Vec<CategoryListItem>,
    pub settings_items: Vec<SettingsItem>,

    pub selected_category_item: usize, // Index in category_list_items
    pub selected_setting: usize,
    pub scroll_offset: usize,
    pub max_completed_setting: usize,

    pub status_message: Option<String>,
    pub status_type: StatusType,
    pub wizard_completed: bool,
    pub editing_setting: bool,
    pub edit_input: String,
    pub edit_cursor_position: usize,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, enhanced_graphics: bool) -> Self {
        let settings_items = Self::create_default_settings();
        let categories = Self::calculate_category_progress(&settings_items);
        let category_list_items = Self::build_category_list(&categories);

        App {
            title,
            should_quit: false,
            screen: Screen::Setup,
            active_panel: ActivePanel::Categories,
            categories,
            category_list_items,
            settings_items,
            selected_category_item: 0,
            selected_setting: 0,
            scroll_offset: 0,
            max_completed_setting: 0,
            status_message: Some(
                "Welcome to Forge of Stories Setup! Navigate with arrow keys, confirm with Enter."
                    .to_string(),
            ),
            status_type: StatusType::Info,
            wizard_completed: false,
            editing_setting: false,
            edit_input: String::new(),
            edit_cursor_position: 0,
        }
    }

    fn create_default_settings() -> Vec<SettingsItem> {
        vec![
            // Network settings - Basic Configuration
            SettingsItem {
                id: "server_port".to_string(),
                name: "Server Port".to_string(),
                description: "Port number for the HTTP server".to_string(),
                category: SettingsCategory::Network,
                subcategory: Some("Basic Configuration".to_string()),
                value: None,
                default_value: "8080".to_string(),
                required: true,
                completed: false,
            },
            SettingsItem {
                id: "server_host".to_string(),
                name: "Server Host".to_string(),
                description: "Host address to bind the server to".to_string(),
                category: SettingsCategory::Network,
                subcategory: Some("Basic Configuration".to_string()),
                value: None,
                default_value: "127.0.0.1".to_string(),
                required: true,
                completed: false,
            },
            // Network settings - SSL/TLS
            SettingsItem {
                id: "ssl_enabled".to_string(),
                name: "Enable SSL/HTTPS".to_string(),
                description: "Enable secure HTTPS connections".to_string(),
                category: SettingsCategory::Network,
                subcategory: Some("SSL/TLS".to_string()),
                value: None,
                default_value: "false".to_string(),
                required: true,
                completed: false,
            },
            SettingsItem {
                id: "ssl_cert_path".to_string(),
                name: "SSL Certificate Path".to_string(),
                description: "Path to the SSL certificate file".to_string(),
                category: SettingsCategory::Network,
                subcategory: Some("SSL/TLS".to_string()),
                value: None,
                default_value: "/etc/ssl/certs/server.crt".to_string(),
                required: false,
                completed: false,
            },
            // Database settings - Connection
            SettingsItem {
                id: "db_url".to_string(),
                name: "Database URL".to_string(),
                description: "Connection string for the database".to_string(),
                category: SettingsCategory::Database,
                subcategory: Some("Connection".to_string()),
                value: None,
                default_value: "sqlite://stories.db".to_string(),
                required: true,
                completed: false,
            },
            SettingsItem {
                id: "db_pool_size".to_string(),
                name: "Connection Pool Size".to_string(),
                description: "Maximum number of database connections".to_string(),
                category: SettingsCategory::Database,
                subcategory: Some("Connection".to_string()),
                value: None,
                default_value: "10".to_string(),
                required: false,
                completed: false,
            },
            // Database settings - Migration
            SettingsItem {
                id: "auto_migrate".to_string(),
                name: "Auto Migration".to_string(),
                description: "Automatically run database migrations on startup".to_string(),
                category: SettingsCategory::Database,
                subcategory: Some("Migration".to_string()),
                value: None,
                default_value: "true".to_string(),
                required: true,
                completed: false,
            },
            // Security settings - Authentication
            SettingsItem {
                id: "jwt_secret".to_string(),
                name: "JWT Secret Key".to_string(),
                description: "Secret key for JWT token signing".to_string(),
                category: SettingsCategory::Security,
                subcategory: Some("Authentication".to_string()),
                value: None,
                default_value: "generate-random-key".to_string(),
                required: true,
                completed: false,
            },
            SettingsItem {
                id: "session_timeout".to_string(),
                name: "Session Timeout".to_string(),
                description: "Session timeout in minutes".to_string(),
                category: SettingsCategory::Security,
                subcategory: Some("Authentication".to_string()),
                value: None,
                default_value: "30".to_string(),
                required: true,
                completed: false,
            },
            // Storage settings - File Upload
            SettingsItem {
                id: "upload_path".to_string(),
                name: "Upload Directory".to_string(),
                description: "Directory for uploaded files".to_string(),
                category: SettingsCategory::Storage,
                subcategory: Some("File Upload".to_string()),
                value: None,
                default_value: "./uploads".to_string(),
                required: true,
                completed: false,
            },
            SettingsItem {
                id: "max_file_size".to_string(),
                name: "Max File Size".to_string(),
                description: "Maximum file size in MB".to_string(),
                category: SettingsCategory::Storage,
                subcategory: Some("File Upload".to_string()),
                value: None,
                default_value: "10".to_string(),
                required: true,
                completed: false,
            },
        ]
    }

    fn calculate_category_progress(settings: &[SettingsItem]) -> Vec<CategoryProgress> {
        let mut progress = Vec::new();

        for category in SettingsCategory::all() {
            let category_settings: Vec<&SettingsItem> = settings
                .iter()
                .filter(|s| s.category == *category)
                .collect();

            // Group by subcategory
            let mut subcategory_map: std::collections::HashMap<String, Vec<&SettingsItem>> =
                std::collections::HashMap::new();

            for setting in &category_settings {
                if let Some(subcategory) = &setting.subcategory {
                    subcategory_map
                        .entry(subcategory.clone())
                        .or_default()
                        .push(setting);
                }
            }

            let mut subcategories = Vec::new();
            for (subcategory_name, subcategory_settings) in subcategory_map {
                let completed_count = subcategory_settings.iter().filter(|s| s.completed).count();
                let total_count = subcategory_settings.len();

                subcategories.push(SubcategoryProgress {
                    name: subcategory_name,
                    completed_count,
                    total_count,
                    is_completed: completed_count == total_count && total_count > 0,
                });
            }

            // Sort subcategories by name for consistent ordering
            subcategories.sort_by(|a, b| a.name.cmp(&b.name));

            let completed_count = category_settings.iter().filter(|s| s.completed).count();
            let total_count = category_settings.len();

            progress.push(CategoryProgress {
                category: *category,
                completed_count,
                total_count,
                is_completed: completed_count == total_count && total_count > 0,
                subcategories,
            });
        }

        progress
    }

    fn build_category_list(categories: &[CategoryProgress]) -> Vec<CategoryListItem> {
        let mut list_items = Vec::new();

        for category_progress in categories {
            // Add main category
            list_items.push(CategoryListItem::Category(
                category_progress.category,
                category_progress.completed_count,
                category_progress.total_count,
                category_progress.is_completed,
            ));

            // Add subcategories (indented)
            for subcategory in &category_progress.subcategories {
                list_items.push(CategoryListItem::Subcategory(
                    category_progress.category,
                    subcategory.name.clone(),
                    subcategory.completed_count,
                    subcategory.total_count,
                    subcategory.is_completed,
                ));
            }
        }

        list_items
    }
    pub fn get_current_category_settings(&self) -> Vec<&SettingsItem> {
        if self.selected_category_item >= self.category_list_items.len() {
            return vec![];
        }

        match &self.category_list_items[self.selected_category_item] {
            CategoryListItem::Category(category, _, _, _) => {
                // Return all settings for this category
                self.settings_items
                    .iter()
                    .filter(|s| s.category == *category)
                    .collect()
            }
            CategoryListItem::Subcategory(category, subcategory_name, _, _, _) => {
                // Return only settings for this specific subcategory
                self.settings_items
                    .iter()
                    .filter(|s| {
                        s.category == *category && s.subcategory.as_ref() == Some(subcategory_name)
                    })
                    .collect()
            }
        }
    }

    pub fn get_total_progress(&self) -> (usize, usize) {
        let completed = self.categories.iter().filter(|c| c.is_completed).count();
        let total = self.categories.len();
        (completed, total)
    }

    pub fn is_setup_complete(&self) -> bool {
        self.categories.iter().all(|c| c.is_completed)
    }

    pub fn on_key(&mut self, key: ratatui::crossterm::event::KeyCode) {
        match (self.screen, self.active_panel) {
            (Screen::Setup, ActivePanel::Categories) => self.handle_category_navigation(key),
            (Screen::Setup, ActivePanel::Settings) => self.handle_settings_navigation(key),
            (Screen::Overview, _) => self.handle_overview_navigation(key),
            _ => {}
        }
    }

    fn handle_category_navigation(&mut self, key: ratatui::crossterm::event::KeyCode) {
        use ratatui::crossterm::event::KeyCode;

        match key {
            KeyCode::Up => {
                if self.selected_category_item > 0 {
                    self.selected_category_item -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected_category_item < self.category_list_items.len().saturating_sub(1) {
                    self.selected_category_item += 1;
                }
            }
            KeyCode::Right | KeyCode::Enter => {
                match &self.category_list_items[self.selected_category_item] {
                    CategoryListItem::Category(category, _, _, _) => {
                        // Check if this category has subcategories
                        let has_subcategories = self
                            .categories
                            .iter()
                            .find(|c| c.category == *category)
                            .map(|c| !c.subcategories.is_empty())
                            .unwrap_or(false);

                        if has_subcategories {
                            // Jump to first subcategory and directly to settings
                            let first_subcategory_index = self.category_list_items.iter()
                                .position(|item| matches!(item, CategoryListItem::Subcategory(cat, _, _, _, _) if cat == category));

                            if let Some(index) = first_subcategory_index {
                                self.selected_category_item = index;

                                // Go directly to settings for this first subcategory
                                let settings = self.get_current_category_settings();
                                if !settings.is_empty() {
                                    self.active_panel = ActivePanel::Settings;
                                    self.selected_setting = 0;
                                    self.max_completed_setting =
                                        self.get_max_completed_setting_for_category();

                                    if let CategoryListItem::Subcategory(
                                        _,
                                        subcategory_name,
                                        _,
                                        _,
                                        _,
                                    ) = &self.category_list_items[index]
                                    {
                                        self.status_message = Some(format!(
                                            "Auto-selected first subcategory: {}. Configure settings below.",
                                            subcategory_name
                                        ));
                                    }
                                }
                                return;
                            }
                        }

                        // No subcategories, go directly to settings
                        let settings = self.get_current_category_settings();
                        if !settings.is_empty() {
                            self.active_panel = ActivePanel::Settings;
                            self.selected_setting = 0;
                            self.max_completed_setting =
                                self.get_max_completed_setting_for_category();
                            self.status_message = Some(format!(
                                "Configure {} settings. Use Enter to confirm values.",
                                category.display_name()
                            ));
                        }
                    }
                    CategoryListItem::Subcategory(_, subcategory_name, _, _, _) => {
                        // Go directly to settings for this subcategory
                        let settings = self.get_current_category_settings();
                        if !settings.is_empty() {
                            self.active_panel = ActivePanel::Settings;
                            self.selected_setting = 0;
                            self.max_completed_setting =
                                self.get_max_completed_setting_for_category();
                            self.status_message = Some(format!(
                                "Configure {} settings. Use Enter to confirm values.",
                                subcategory_name
                            ));
                        }
                    }
                }
            }
            KeyCode::Char('q') => {
                if !self.wizard_completed {
                    panic!(
                        "Wizard wurde vorzeitig beendet! Alle Einstellungen müssen abgeschlossen werden."
                    );
                }
                self.should_quit = true;
            }
            _ => {}
        }
    }

    fn handle_settings_navigation(&mut self, key: ratatui::crossterm::event::KeyCode) {
        use ratatui::crossterm::event::KeyCode;

        if self.editing_setting {
            self.handle_edit_input(key);
            return;
        }

        let settings = self.get_current_category_settings();

        match key {
            KeyCode::Up => {
                if self.selected_setting > 0 {
                    self.selected_setting -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected_setting < settings.len().saturating_sub(1) {
                    self.selected_setting += 1;
                }
            }
            KeyCode::Left => {
                self.active_panel = ActivePanel::Categories;
                self.status_message =
                    Some("Select a category to configure its settings.".to_string());
            }
            KeyCode::Enter => {
                self.confirm_current_setting();
            }
            KeyCode::Char('e') => {
                self.start_editing_setting();
            }
            KeyCode::Char('q') => {
                if !self.wizard_completed {
                    panic!(
                        "Wizard wurde vorzeitig beendet! Alle Einstellungen müssen abgeschlossen werden."
                    );
                }
                self.should_quit = true;
            }
            _ => {}
        }
    }

    fn handle_overview_navigation(&mut self, key: ratatui::crossterm::event::KeyCode) {
        use ratatui::crossterm::event::KeyCode;

        match key {
            KeyCode::Enter => {
                // Save configuration and exit
                self.mark_wizard_completed();
                self.should_quit = true;
            }
            KeyCode::Esc => {
                self.screen = Screen::Setup;
                self.active_panel = ActivePanel::Categories;
            }
            KeyCode::Char('q') => {
                if !self.wizard_completed {
                    panic!(
                        "Wizard wurde vorzeitig beendet! Alle Einstellungen müssen abgeschlossen werden."
                    );
                }
                self.should_quit = true;
            }
            _ => {}
        }
    }

    fn get_max_completed_setting_for_category(&self) -> usize {
        let settings = self.get_current_category_settings();
        let mut max_index = 0;

        for (i, setting) in settings.iter().enumerate() {
            if setting.completed {
                max_index = i + 1;
            } else {
                break;
            }
        }

        max_index
    }

    fn confirm_current_setting(&mut self) {
        if self.selected_category_item >= self.category_list_items.len() {
            return;
        }

        // Get the current settings (which are already filtered by category/subcategory)
        let current_settings = self.get_current_category_settings();
        if self.selected_setting >= current_settings.len() {
            return;
        }

        // Find the global index of the currently selected setting
        let target_setting_id = &current_settings[self.selected_setting].id;
        let target_index = self
            .settings_items
            .iter()
            .position(|s| s.id == *target_setting_id);

        if let Some(idx) = target_index {
            // Update the setting without causing multiple mutable borrows
            if !self.settings_items[idx].completed {
                {
                    let setting = &mut self.settings_items[idx];
                    // Use default value for now - in a real app you'd have an input field
                    setting.value = Some(setting.default_value.clone());
                    setting.completed = true;
                }

                // Update max completed setting
                self.max_completed_setting =
                    self.max_completed_setting.max(self.selected_setting + 1);

                // Update category progress immediately to refresh counters
                self.update_category_progress();
            }

            // Move to next setting if available
            let settings_count = self.get_current_category_settings().len();
            if self.selected_setting < settings_count.saturating_sub(1) {
                self.selected_setting += 1;
            } else {
                // Current category/subcategory completed, return to category view
                self.active_panel = ActivePanel::Categories;
                self.update_category_progress();

                // Check if all setup is complete
                if self.is_setup_complete() {
                    self.screen = Screen::Overview;
                    self.status_message =
                        Some("🎉 Setup complete! Review your configuration below.".to_string());
                    self.status_type = StatusType::Success;
                } else {
                    let completion_message =
                        match &self.category_list_items[self.selected_category_item] {
                            CategoryListItem::Category(category, _, _, _) => {
                                format!(
                                    "✔ {} completed! Moving to next category.",
                                    category.display_name()
                                )
                            }
                            CategoryListItem::Subcategory(_, subcategory_name, _, _, _) => {
                                format!("✔ {} completed! Moving to next item.", subcategory_name)
                            }
                        };
                    self.status_message = Some(completion_message);
                    self.status_type = StatusType::Success;

                    // Auto-navigate to next category/subcategory
                    self.move_to_next_incomplete_item();
                }
            }
        }
    }

    fn update_category_progress(&mut self) {
        self.categories = Self::calculate_category_progress(&self.settings_items);
        self.category_list_items = Self::build_category_list(&self.categories);
    }

    fn move_to_next_incomplete_item(&mut self) {
        // Start from the next item after current
        let start_index = (self.selected_category_item + 1) % self.category_list_items.len();

        for i in 0..self.category_list_items.len() {
            let index = (start_index + i) % self.category_list_items.len();

            match &self.category_list_items[index] {
                CategoryListItem::Category(_, _, _, is_completed) => {
                    if !is_completed {
                        self.selected_category_item = index;
                        return;
                    }
                }
                CategoryListItem::Subcategory(_, _, _, _, is_completed) => {
                    if !is_completed {
                        self.selected_category_item = index;
                        return;
                    }
                }
            }
        }
    }

    pub fn on_tick(&mut self) {
        // Update progress
    }

    pub fn is_wizard_completed(&self) -> bool {
        self.settings_items.iter().all(|item| item.completed)
    }

    pub fn mark_wizard_completed(&mut self) {
        self.wizard_completed = true;
    }

    fn start_editing_setting(&mut self) {
        let settings = self.get_current_category_settings();
        if self.selected_setting < settings.len() {
            let setting = &settings[self.selected_setting];
            let setting_name = setting.name.clone();
            let setting_value = setting
                .value
                .clone()
                .unwrap_or_else(|| setting.default_value.clone());

            self.edit_input = setting_value;
            self.edit_cursor_position = self.edit_input.chars().count();
            self.editing_setting = true;
            self.status_message = Some(format!(
                "Editing '{}'. Press Enter to confirm, Esc to cancel.",
                setting_name
            ));
        }
    }

    fn handle_edit_input(&mut self, key: ratatui::crossterm::event::KeyCode) {
        use ratatui::crossterm::event::KeyCode;

        match key {
            KeyCode::Enter => {
                self.confirm_edit();
            }
            KeyCode::Esc => {
                self.cancel_edit();
            }
            KeyCode::Left => {
                self.move_cursor_left();
            }
            KeyCode::Right => {
                self.move_cursor_right();
            }
            KeyCode::Backspace => {
                self.delete_char();
            }
            KeyCode::Char(c) => {
                self.enter_char(c);
            }
            _ => {}
        }
    }

    fn confirm_edit(&mut self) {
        let settings = self.get_current_category_settings();
        if self.selected_setting < settings.len() {
            let setting_id = settings[self.selected_setting].id.clone();

            // Find and update the setting in settings_items
            for setting in &mut self.settings_items {
                if setting.id == setting_id {
                    setting.value = Some(self.edit_input.clone());
                    setting.completed = true;
                    break;
                }
            }

            self.update_category_progress();
        }

        self.editing_setting = false;
        self.edit_input.clear();
        self.edit_cursor_position = 0;
        self.status_message = Some("Setting updated successfully!".to_string());
    }

    fn cancel_edit(&mut self) {
        self.editing_setting = false;
        self.edit_input.clear();
        self.edit_cursor_position = 0;
        self.status_message = Some("Edit cancelled.".to_string());
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.edit_cursor_position.saturating_sub(1);
        self.edit_cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.edit_cursor_position.saturating_add(1);
        self.edit_cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.edit_input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.edit_input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.edit_cursor_position)
            .unwrap_or(self.edit_input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.edit_cursor_position != 0;
        if is_not_cursor_leftmost {
            let current_index = self.edit_cursor_position;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.edit_input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.edit_input.chars().skip(current_index);

            self.edit_input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.edit_input.chars().count())
    }
}
