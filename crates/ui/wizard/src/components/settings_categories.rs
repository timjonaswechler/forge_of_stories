use color_eyre::Result;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::{action::Action, components::Component};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    General,
    Network,
    Security,
    Monitoring,
    Uds,
}

impl Category {
    pub fn all() -> Vec<Category> {
        vec![
            Category::General,
            Category::Network,
            Category::Security,
            Category::Monitoring,
            Category::Uds,
        ]
    }
    pub fn title(&self) -> &'static str {
        match self {
            Category::General => "General",
            Category::Network => "Network",
            Category::Security => "Security",
            Category::Monitoring => "Monitoring",
            Category::Uds => "UDS",
        }
    }
}

pub struct SettingsCategoriesComponent {
    categories: Vec<Category>,
    state: ListState,
}

impl SettingsCategoriesComponent {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            categories: Category::all(),
            state,
        }
    }

    pub fn selected(&self) -> Category {
        let idx = self.state.selected().unwrap_or(0);
        self.categories
            .get(idx)
            .copied()
            .unwrap_or(Category::General)
    }

    fn select_up(&mut self) {
        let i = self.state.selected().unwrap_or(0);
        self.state.select(Some(i.saturating_sub(1)));
    }
    fn select_down(&mut self) {
        let i = self.state.selected().unwrap_or(0);
        let max = self.categories.len().saturating_sub(1);
        self.state.select(Some((i + 1).min(max)));
    }
}

impl Component for SettingsCategoriesComponent {
    fn height_constraint(&self) -> Constraint {
        Constraint::Fill(1)
    }

    fn name(&self) -> &'static str {
        "settings_left"
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Up => self.select_up(),
            Action::Down => self.select_down(),
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let items: Vec<ListItem> = self
            .categories
            .iter()
            .map(|c| ListItem::new(c.title()))
            .collect();

        let list = List::new(items)
            .block(Block::default().title("Kategorien").borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED));

        f.render_stateful_widget(list, area, &mut self.state);
        Ok(())
    }
}
