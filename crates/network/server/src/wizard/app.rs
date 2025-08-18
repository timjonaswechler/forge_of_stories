use super::ui;
use anyhow::anyhow;
use ratatui::{
    Terminal,
    backend::Backend,
    crossterm::event::{self, Event, KeyEvent, KeyEventKind},
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::{
    io,
    time::{Duration, Instant},
};
use tui_textarea::TextArea;

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
    Security,
    World,
    Features,
    Finished,
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
            "security" => SettingsCategory::Security,
            "world" => SettingsCategory::World,
            "features" => SettingsCategory::Features,
            "finished" => SettingsCategory::Finished,
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
            SettingsCategory::Security => "security",
            SettingsCategory::World => "world",
            SettingsCategory::Features => "features",
            SettingsCategory::Finished => "finished",
        }
    }

    #[must_use]
    pub const fn all() -> &'static [SettingsCategory] {
        &[
            SettingsCategory::Network,
            SettingsCategory::Security,
            SettingsCategory::World,
            SettingsCategory::Features,
        ]
    }

    #[must_use]
    pub const fn all_without_finished() -> &'static [SettingsCategory] {
        Self::all()
    }

    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            SettingsCategory::Network => "Network Configuration",
            SettingsCategory::Security => "Security & Authentication",
            SettingsCategory::World => "World Settings",
            SettingsCategory::Features => "Feature Flags",
            SettingsCategory::Finished => "✓ Fertig - Einstellungen Bestätigen",
        }
    }

    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            SettingsCategory::Network => "Configure server ports, SSL, and network protocols",
            SettingsCategory::Security => {
                "Authentication methods, encryption keys, and permissions"
            }
            SettingsCategory::World => "Settings for world generation, dimensions, and features",
            SettingsCategory::Features => "Enable or disable experimental features and modules",
            SettingsCategory::Finished => {
                "Alle Einstellungen abgeschlossen - Server starten und Konfiguration speichern"
            }
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

pub struct WizardApp<'a> {
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
    pub edit_scroll_offset: u16,
    // TextArea instance used while editing. When Some, TextArea holds the editor state.
    pub edit_textarea: Option<TextArea<'a>>,
}

impl<'a> WizardApp<'a> {
    pub fn run<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        tick_duration: Duration,
    ) -> io::Result<()> {
        let mut last_tick = Instant::now();
        loop {
            // pass the mutable reference `self` (which is of type `&mut App<'a>`) directly to the UI draw fn
            // (using `&mut self` would create a `&mut &mut App` which is incorrect).
            let draw_result = terminal.draw(|frame| ui::draw(frame, self));
            if let Err(e) = draw_result {
                return Err(e);
            }

            let timeout = tick_duration.saturating_sub(last_tick.elapsed());
            if let Ok(true) = event::poll(timeout) {
                if let Ok(Event::Key(key)) = event::read() {
                    if key.kind == KeyEventKind::Press {
                        self.on_key(KeyEvent {
                            kind: key.kind,
                            state: key.state,
                            code: key.code,
                            modifiers: key.modifiers,
                        });
                    }
                }
            }
            if last_tick.elapsed() >= tick_duration {
                last_tick = Instant::now();
            }
            if self.should_quit {
                return Ok(());
            }
        }
    }

    pub fn new(title: &'a str) -> Self {
        let settings_items = Self::create_default_settings();
        let categories = Self::calculate_category_progress(&settings_items);
        let category_list_items = Self::build_category_list(&categories);

        WizardApp {
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
            edit_scroll_offset: 0,
            edit_textarea: None,
        }
    }

    fn create_default_settings() -> Vec<SettingsItem> {
        vec![
            // Network settings - Basic Configuration
            SettingsItem {
                id: "quic_port".to_string(),
                name: "QUIC Gaming Port".to_string(),
                description: "Port number for the QUIC server".to_string(),
                category: SettingsCategory::Network,
                subcategory: Some("Basic Configuration".to_string()),
                value: None,
                default_value: "8080".to_string(),
                required: true,
                completed: false,
            },
            SettingsItem {
                id: "admin_port".to_string(),
                name: "Admin Port".to_string(),
                description: "Port number for the Administration of the server".to_string(),
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

        // Add "Finished" category only if all other settings are completed
        let all_settings_completed = settings.iter().all(|s| s.completed);
        if all_settings_completed {
            progress.push(CategoryProgress {
                category: SettingsCategory::Finished,
                completed_count: 0,
                total_count: 0,
                is_completed: false,
                subcategories: vec![],
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
                if *category == SettingsCategory::Finished {
                    // Finished category has no settings
                    vec![]
                } else {
                    // Return all settings for this category
                    self.settings_items
                        .iter()
                        .filter(|s| s.category == *category)
                        .collect()
                }
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

    // Now accepts the full KeyEvent so we can forward events to TextArea while editing.
    pub fn on_key(&mut self, key_event: ratatui::crossterm::event::KeyEvent) {
        match (self.screen, self.active_panel) {
            (Screen::Setup, ActivePanel::Categories) => {
                // Category navigation still uses KeyCode only.
                self.handle_category_navigation(key_event.code)
            }
            (Screen::Setup, ActivePanel::Settings) => {
                if self.editing_setting {
                    // While editing, forward whole KeyEvent to TextArea / edit handler.
                    self.handle_edit_input(key_event);
                } else {
                    self.handle_settings_navigation(key_event.code);
                }
            }
            (Screen::Overview, _) => self.handle_overview_navigation(key_event.code),
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
                        // Handle "Finished" category specially
                        if *category == SettingsCategory::Finished {
                            if self.is_wizard_completed() {
                                self.mark_wizard_completed();
                                self.should_quit = true;
                                self.status_message = Some(
                                    "✓ Alle Einstellungen bestätigt! Server wird gestartet..."
                                        .to_string(),
                                );
                                return;
                            } else {
                                self.status_message = Some(
                                    "⚠ Nicht alle Einstellungen sind abgeschlossen!".to_string(),
                                );
                                return;
                            }
                        }
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
            self.handle_edit_input(key.into());
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

            // Populate both the plain string and create a TextArea instance for editing.
            self.edit_input = setting_value.clone();
            self.edit_cursor_position = self.edit_input.chars().count();

            // Create a TextArea from current value; split by commas into lines so each list item appears on its own line.
            // Use owned Strings to satisfy the TextArea::from iterator requirement.
            let lines_iter = setting_value.split(',').map(|s| s.trim().to_string());
            let mut textarea = TextArea::from(lines_iter);
            // Optionally configure textarea (no special config here).
            self.edit_textarea = Some(textarea);

            self.editing_setting = true;
            self.status_message = Some(format!(
                "Editing '{}'. Press Enter to confirm, Esc to cancel.",
                setting_name
            ));
        }
    }

    fn handle_edit_input(&mut self, key_event: ratatui::crossterm::event::KeyEvent) {
        use ratatui::crossterm::event::KeyCode;

        // Intercept certain keys (Enter, Esc, comma) to implement the desired behavior.
        // Otherwise, forward the event to the TextArea instance.
        match key_event.code {
            KeyCode::Enter => {
                // Confirm and close edit session
                self.confirm_edit();
            }
            KeyCode::Esc => {
                self.cancel_edit();
            }
            KeyCode::Char(',') => {
                // Insert comma into textarea and then convert it into a new list entry (newline).
                if let Some(textarea) = &mut self.edit_textarea {
                    // First let the textarea insert the comma character
                    textarea.input(key_event);
                    // Then insert a newline right after comma so next entry is on its own line.
                    textarea.insert_newline();
                } else {
                    // Fallback: operate on raw edit_input
                    self.enter_char(',');
                }
            }
            other => {
                if let Some(textarea) = &mut self.edit_textarea {
                    // Forward the entire KeyEvent to the TextArea for default behavior.
                    textarea.input(key_event);
                } else {
                    // Fallback to legacy single-line editing helpers
                    match other {
                        KeyCode::Left => self.move_cursor_left(),
                        KeyCode::Right => self.move_cursor_right(),
                        KeyCode::Backspace => self.delete_char(),
                        KeyCode::Char(c) => self.enter_char(c),
                        _ => {}
                    }
                }
            }
        }
    }

    fn confirm_edit(&mut self) {
        let settings = self.get_current_category_settings();
        if self.selected_setting < settings.len() {
            let setting_id = settings[self.selected_setting].id.clone();

            // Determine final value either from TextArea (preferred) or fallback to edit_input.
            let final_value = if let Some(textarea) = self.edit_textarea.take() {
                // consume textarea and join lines with commas (the app expects comma-separated values)
                let lines: Vec<String> = textarea.into_lines();
                lines
                    .into_iter()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join(",")
            } else {
                self.edit_input.clone()
            };

            // Find and update the setting in settings_items
            for setting in &mut self.settings_items {
                if setting.id == setting_id {
                    setting.value = Some(final_value);
                    setting.completed = true;
                    break;
                }
            }

            self.update_category_progress();
        }

        self.editing_setting = false;
        self.edit_input.clear();
        self.edit_cursor_position = 0;
        self.edit_scroll_offset = 0;
        self.edit_textarea = None;
        self.status_message = Some("Setting updated successfully!".to_string());
    }

    fn cancel_edit(&mut self) {
        // Discard textarea state and revert to empty edit state
        self.editing_setting = false;
        self.edit_input.clear();
        self.edit_cursor_position = 0;
        self.edit_scroll_offset = 0;
        self.edit_textarea = None;
        self.status_message = Some("Edit cancelled.".to_string());
    }

    fn move_cursor_left(&mut self) {
        if self.edit_cursor_position > 0 {
            self.edit_cursor_position = self.edit_cursor_position.saturating_sub(1);
            self.update_edit_scroll(5);
        }
    }

    fn move_cursor_right(&mut self) {
        if self.edit_cursor_position < self.edit_input.chars().count() {
            self.edit_cursor_position = self.edit_cursor_position.saturating_add(1);
            self.update_edit_scroll(5);
        }
    }

    fn move_cursor_up(&mut self) {
        if !self.edit_input.contains(',') {
            return;
        }

        let parts: Vec<&str> = self.edit_input.split(',').collect();

        // compute start indices (char offsets) for each part
        let mut starts: Vec<usize> = Vec::with_capacity(parts.len());
        let mut pos = 0usize;
        for (i, p) in parts.iter().enumerate() {
            starts.push(pos);
            pos += p.chars().count();
            if i < parts.len() - 1 {
                pos += 1; // comma
            }
        }

        // find current part index
        let mut cur_idx: Option<usize> = None;
        for (i, &start) in starts.iter().enumerate() {
            let len = parts[i].chars().count();
            if self.edit_cursor_position >= start && self.edit_cursor_position <= start + len {
                cur_idx = Some(i);
                break;
            }
        }

        let cur_idx = match cur_idx {
            Some(i) => i,
            None => return,
        };

        if cur_idx == 0 {
            return;
        }

        let cur_start = starts[cur_idx];
        let offset = self.edit_cursor_position.saturating_sub(cur_start);
        let prev_idx = cur_idx - 1;
        let prev_start = starts[prev_idx];
        let prev_len = parts[prev_idx].chars().count();
        let new_offset = offset.min(prev_len);
        self.edit_cursor_position = prev_start + new_offset;
        self.update_edit_scroll(5);
    }

    fn move_cursor_down(&mut self) {
        if !self.edit_input.contains(',') {
            return;
        }

        let parts: Vec<&str> = self.edit_input.split(',').collect();

        // compute start indices (char offsets) for each part
        let mut starts: Vec<usize> = Vec::with_capacity(parts.len());
        let mut pos = 0usize;
        for (i, p) in parts.iter().enumerate() {
            starts.push(pos);
            pos += p.chars().count();
            if i < parts.len() - 1 {
                pos += 1; // comma
            }
        }

        // find current part index
        let mut cur_idx: Option<usize> = None;
        for (i, &start) in starts.iter().enumerate() {
            let len = parts[i].chars().count();
            if self.edit_cursor_position >= start && self.edit_cursor_position <= start + len {
                cur_idx = Some(i);
                break;
            }
        }

        let cur_idx = match cur_idx {
            Some(i) => i,
            None => return,
        };

        if cur_idx + 1 >= parts.len() {
            return;
        }

        let cur_start = starts[cur_idx];
        let offset = self.edit_cursor_position.saturating_sub(cur_start);
        let next_idx = cur_idx + 1;
        let next_start = starts[next_idx];
        let next_len = parts[next_idx].chars().count();
        let new_offset = offset.min(next_len);
        self.edit_cursor_position = next_start + new_offset;
        self.update_edit_scroll(5);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.edit_input.insert(index, new_char);
        // After inserting, advance cursor by one character (char index)
        self.edit_cursor_position = self.edit_cursor_position.saturating_add(1);
        self.update_edit_scroll(5);
    }

    fn byte_index(&self) -> usize {
        if self.edit_cursor_position == 0 {
            return 0;
        }
        // Map character index to byte index
        self.edit_input
            .char_indices()
            .nth(self.edit_cursor_position)
            .map(|(byte_idx, _)| byte_idx)
            .unwrap_or_else(|| self.edit_input.len())
    }

    fn delete_char(&mut self) {
        if self.edit_cursor_position == 0 {
            return;
        }
        // Remove the character before the cursor using a char buffer to avoid byte-index issues
        let mut chars: Vec<char> = self.edit_input.chars().collect();
        let remove_index = self.edit_cursor_position.saturating_sub(1);
        if remove_index < chars.len() {
            chars.remove(remove_index);
            self.edit_input = chars.into_iter().collect();
            self.edit_cursor_position = remove_index;
        }
        self.update_edit_scroll(5);
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.edit_input.chars().count())
    }

    pub fn update_edit_scroll(&mut self, visible_lines: u16) {
        if !self.editing_setting {
            return;
        }

        // Calculate which line the cursor is on
        let cursor_line = self.get_cursor_line();

        // Adjust scroll if cursor is outside visible area
        if cursor_line < self.edit_scroll_offset {
            self.edit_scroll_offset = cursor_line;
        } else if cursor_line >= self.edit_scroll_offset + visible_lines {
            self.edit_scroll_offset = cursor_line.saturating_sub(visible_lines) + 1;
        }
    }

    fn get_cursor_line(&self) -> u16 {
        if self.edit_input.is_empty() {
            return 0;
        }

        let parts: Vec<&str> = self.edit_input.split(',').collect();
        let mut char_count = 0usize;

        for (i, part) in parts.iter().enumerate() {
            let len = part.chars().count();
            if self.edit_cursor_position >= char_count
                && self.edit_cursor_position <= char_count + len
            {
                return i as u16;
            }
            char_count += len;
            if i < parts.len() - 1 {
                char_count += 1; // comma
            }
        }

        // Fallback to last line
        (parts.len().saturating_sub(1)) as u16
    }
}
