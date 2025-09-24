use crate::{
    action::Action,
    components::{Component, ComponentKey},
    layers::ActionOutcome,
};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

/// Small info panel used for quick hints on the welcome/dashboard pages.
pub(crate) struct Info {
    id: Option<ComponentKey>,
    title: String,
    body: Vec<String>,
}

impl Info {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: None,
            title: title.into(),
            body: Vec::new(),
        }
    }

    pub fn add_line(mut self, line: impl Into<String>) -> Self {
        self.body.push(line.into());
        self
    }
}

impl Component for Info {
    fn name(&self) -> &str {
        "info"
    }

    fn id(&self) -> ComponentKey {
        self.id.expect("Component ID not set")
    }

    fn set_id(&mut self, id: ComponentKey) {
        self.id = Some(id);
    }

    fn focusable(&self) -> bool {
        false
    }
    fn kind(&self) -> &'static str {
        "info"
    }

    fn handle_action(&mut self, _action: &Action) -> ActionOutcome {
        ActionOutcome::NotHandled
    }

    fn render(&self, f: &mut ratatui::Frame, area: Rect) {
        let mut width = self.title.char_indices().count();
        let block = Block::new().title(self.title.clone()).title_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

        let par = if !self.body.is_empty() {
            let lines: Vec<Line> = self
                .body
                .iter()
                .map(|line| Line::from(Span::raw(line.clone())))
                .collect();
            for elem in self.body.iter() {
                if elem.len() > width {
                    width = elem.len();
                }
            }

            Paragraph::new(lines).block(block).wrap(Wrap { trim: true })
        } else {
            Paragraph::new("").block(block)
        };
        let center = Layout::horizontal([Constraint::Length(width as u16)])
            .flex(Flex::Center)
            .areas::<1>(area)[0];
        f.render_widget(par, center);
    }
}
