use crate::{
    action::{Action, UiAction},
    components::Component,
};
use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
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
    focused: bool,
    selected: usize,
    items: Vec<StatusItemKind>,
    snapshot: StatusSnapshot,
}

impl AetherStatusListComponent {
    pub(crate) fn new() -> Self {
        Self {
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

    fn popup_id(&self) -> &'static str {
        match self.items[self.selected] {
            StatusItemKind::Settings => "settings_popup",
            StatusItemKind::Certificate => "certificate_popup",
            StatusItemKind::Uds => "uds_popup",
        }
    }

    fn draw_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
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
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    let text_style = if sel {
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
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    let text_style = if sel {
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
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    let text_style = if sel {
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
    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        use crossterm::event::KeyCode::*;
        match key.code {
            Up => {
                self.handle_nav(-1);
                return Ok(None);
            }
            Down => {
                self.handle_nav(1);
                return Ok(None);
            }
            Char('k') => {
                self.handle_nav(-1);
                return Ok(None);
            }
            Char('j') => {
                self.handle_nav(1);
                return Ok(None);
            }
            Enter => {
                // Open popup for selected item
                return Ok(Some(Action::Ui(UiAction::OpenPopup {
                    id: self.popup_id().to_string(),
                    priority: None,
                })));
            }
            _ => {}
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::Tick = action {
            self.on_tick();
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, body: Rect) -> Result<()> {
        let [area] = Layout::horizontal([Constraint::Length(28)])
            .flex(Flex::Center)
            .areas(body);
        let lines = self.draw_lines();
        let paragraph = Paragraph::new(lines).style(Style::default());
        f.render_widget(paragraph, area);
        Ok(())
    }
}
