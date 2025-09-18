use crate::{
    action::{Action, UiAction},
    layers::ActionOutcome,
    ui::components::{Component, ComponentKey},
};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

#[derive(Clone, Copy, Debug)]
enum StatusItemKind {
    Settings,
    Certificate,
    Uds,
}

struct StatusSnapshot {
    settings_found: bool,
    settings_valid: bool,
    cert_found: bool,
    uds_found: bool,
}

impl StatusSnapshot {
    fn probe() -> Self {
        Self {
            settings_found: aether_config::settings_found(),
            settings_valid: aether_config::settings_valid(),
            cert_found: aether_config::certificate_found(),
            uds_found: aether_config::uds_found(),
        }
    }
}

pub(crate) struct AetherStatusListComponent {
    id: Option<ComponentKey>,
    focused: bool,
    selected: usize,
    items: Vec<StatusItemKind>,
    snapshot: StatusSnapshot,
}

impl AetherStatusListComponent {
    pub(crate) fn new() -> Self {
        Self {
            id: None,
            focused: false,
            selected: 0,
            items: vec![
                StatusItemKind::Settings,
                StatusItemKind::Certificate,
                StatusItemKind::Uds,
            ],
            snapshot: StatusSnapshot::probe(),
        }
    }

    fn on_tick(&mut self) {
        // Re-probe; later we might debounce or compare.
        self.snapshot = StatusSnapshot::probe();
    }

    fn handle_nav(&mut self, dir: i32) {
        let len = self.items.len();
        if len == 0 {
            return;
        }
        let cur = self.selected as i32;
        let next = (cur + dir).rem_euclid(len as i32);
        self.selected = next as usize;
    }

    fn draw_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let focus_accent = if self.focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };
        let focus_bold = if self.focused {
            Modifier::BOLD
        } else {
            Modifier::empty()
        };
        for (idx, kind) in self.items.iter().enumerate() {
            match kind {
                StatusItemKind::Settings => {
                    let sel = idx == self.selected;
                    let (found, valid) =
                        (self.snapshot.settings_found, self.snapshot.settings_valid);

                    let marker: Vec<Span> = if found {
                        if valid {
                            vec![
                                " [  ".into(),
                                Span::styled("ok", Style::default().fg(Color::Green)),
                                "  ] ".into(),
                            ]
                        } else {
                            vec![
                                " [ ".into(),
                                Span::styled("failed", Style::default().fg(Color::Red)),
                                " ] ".into(),
                            ]
                        }
                    } else {
                        vec![
                            " [ ".into(),
                            Span::styled("      ", Style::default().fg(Color::Gray)),
                            " ] ".into(),
                        ]
                    };
                    let arrow_style = if sel {
                        Style::default()
                            .fg(focus_accent)
                            .add_modifier(focus_bold)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    let text_style = if sel && self.focused {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    let mut line_spans =
                        vec![Span::styled(if sel { "> " } else { "  " }, arrow_style)];
                    line_spans.extend(marker);
                    line_spans.push(Span::styled(" Aether Settings ", text_style));
                    lines.push(Line::from(line_spans));
                }
                StatusItemKind::Certificate => {
                    let sel = idx == self.selected;
                    let found = self.snapshot.cert_found;
                    let marker: Vec<Span> = if found {
                        vec![
                            " [  ".into(),
                            Span::styled("ok", Style::default().fg(Color::Green)),
                            "  ] ".into(),
                        ]
                    } else {
                        vec![
                            " [ ".into(),
                            Span::styled("    ", Style::default().fg(Color::Gray)),
                            " ] ".into(),
                        ]
                    };
                    let arrow_style = if sel {
                        Style::default()
                            .fg(focus_accent)
                            .add_modifier(focus_bold)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    let text_style = if sel && self.focused {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    let mut line_spans =
                        vec![Span::styled(if sel { "> " } else { "  " }, arrow_style)];
                    line_spans.extend(marker);
                    line_spans.push(Span::styled(" Certificate ", text_style));
                    lines.push(Line::from(line_spans));
                }
                StatusItemKind::Uds => {
                    let sel = idx == self.selected;
                    let found = self.snapshot.uds_found;
                    let marker: Vec<Span> = if found {
                        vec![
                            " [  ".into(),
                            Span::styled("ok", Style::default().fg(Color::Green)),
                            "  ] ".into(),
                        ]
                    } else {
                        vec![
                            " [ ".into(),
                            Span::styled("    ", Style::default().fg(Color::Gray)),
                            " ] ".into(),
                        ]
                    };
                    let arrow_style = if sel {
                        Style::default()
                            .fg(focus_accent)
                            .add_modifier(focus_bold)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    let text_style = if sel && self.focused {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    let mut line_spans =
                        vec![Span::styled(if sel { "> " } else { "  " }, arrow_style)];
                    line_spans.extend(marker);
                    line_spans.push(Span::styled(" UDS Socket  ", text_style));
                    lines.push(Line::from(line_spans));
                }
            }
            // Blank spacer line after each group except last (for readability)
            if idx != self.items.len() - 1 {
                lines.push(Line::raw(""));
            }
        }
        lines
    }
}

impl Component for AetherStatusListComponent {
    fn name(&self) -> &str {
        "aether_status"
    }

    fn id(&self) -> ComponentKey {
        self.id.expect("Component ID not set")
    }

    fn set_id(&mut self, id: ComponentKey) {
        self.id = Some(id);
    }

    fn on_focus(&mut self, gained: bool) {
        self.focused = gained;
    }

    fn handle_action(&mut self, action: &Action) -> ActionOutcome {
        match action {
            Action::Ui(UiAction::NavigateUp) => {
                self.handle_nav(-1);
                ActionOutcome::Consumed
            }
            Action::Ui(UiAction::NavigateDown) => {
                self.handle_nav(1);
                ActionOutcome::Consumed
            }
            Action::Tick => {
                self.on_tick();
                ActionOutcome::Consumed
            }
            _ => ActionOutcome::NotHandled,
        }
    }

    fn render(&self, f: &mut ratatui::Frame, body: Rect) {
        let [area] = Layout::horizontal([Constraint::Length(28)])
            .flex(Flex::Center)
            .areas(body);
        let lines = self.draw_lines();
        let paragraph = Paragraph::new(lines).style(Style::default());
        f.render_widget(paragraph, area);
    }
}
