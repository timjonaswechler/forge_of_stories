use crate::{
    action::Action,
    layers::ActionOutcome,
    ui::components::{Component, ComponentKey},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};
use std::collections::HashMap;

/// Decorative logo block rendered on the welcome/dashboard pages.
pub(crate) struct Logo {
    id: Option<ComponentKey>,
    label: String,
}

impl Logo {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
        }
    }
    pub fn wizard_lines(&self) -> Vec<String> {
        vec![
            "                                               ".to_string(),
            "██        ██ ██ ███████  █████  ██████  ██████ ".to_string(),
            "██   ██   ██ ██      ██ ██   ██ ██   ██ ██   ██".to_string(),
            " ██ ████ ██  ██   ███   ███████ ██████  ██   ██".to_string(),
            " ████  ████  ██ ██      ██   ██ ██   ██ ██   ██".to_string(),
            "  ██    ██   ██ ███████ ██   ██ ██   ██ ██████ ".to_string(),
            "                                               ".to_string(),
        ]
    }

    pub fn wizard_color(&self) -> Vec<String> {
        vec![
            "                                               ".to_string(),
            "AA        DD EE EFFFGGG  HHIII  JJKKKL  LMMMNN ".to_string(),
            "AA   BC   DD EE      GG HH   IJ JJ   LL LM   NN".to_string(),
            " AA BBCC DD  EE   FFG   HHHIIIJ JJKKKL  LM   NN".to_string(),
            " AABB  CCDD  EE EF      HH   IJ JJ   LL LM   NN".to_string(),
            "  AB    CD   EE EFFFGGG HH   IJ JJ   LL LMMMNN ".to_string(),
            "                                               ".to_string(),
        ]
    }
    pub fn wizard_colormap(&self) -> HashMap<char, Color> {
        let mut color_map = HashMap::new();
        color_map.insert('A', Color::Rgb(91, 0, 130));
        color_map.insert('B', Color::Rgb(85, 1, 129));
        color_map.insert('C', Color::Rgb(68, 3, 127));
        color_map.insert('D', Color::Rgb(54, 4, 126));
        color_map.insert('E', Color::Rgb(38, 6, 124));
        color_map.insert('F', Color::Rgb(30, 7, 123));
        color_map.insert('G', Color::Rgb(20, 8, 122));
        color_map.insert('H', Color::Rgb(13, 9, 121));
        color_map.insert('I', Color::Rgb(8, 17, 129));
        color_map.insert('J', Color::Rgb(7, 38, 151));
        color_map.insert('K', Color::Rgb(5, 64, 178));
        color_map.insert('L', Color::Rgb(3, 89, 203));
        color_map.insert('M', Color::Rgb(1, 116, 230));
        color_map.insert('N', Color::Rgb(0, 140, 255));
        color_map
    }
    pub fn fos_lines(&self) -> Vec<String> {
        vec![
            "███████████                                                     ██████ ".to_string(),
            " ███      █                                                    ███  ███".to_string(),
            " ███   █     ██████  ████████   ███████  ██████      ██████    ███     ".to_string(),
            " ███████    ███  ███  ███  ███ ███  ███ ███  ███    ███  ███ ███████   ".to_string(),
            " ███   █    ███  ███  ███      ███  ███ ███████     ███  ███   ███     ".to_string(),
            " ███        ███  ███  ███      ███  ███ ███         ███  ███   ███     ".to_string(),
            "█████        ██████  █████      ███████  ██████      ██████   █████    ".to_string(),
            "                                    ███                                ".to_string(),
            "                               ███  ███                                ".to_string(),
            "    █████████    ███            ██████       ███                       ".to_string(),
            "   ███     ███   ███                                                   ".to_string(),
            "   ███         ███████    ██████  ████████  ████   ██████   █████      ".to_string(),
            "    █████████    ███     ███  ███  ███  ███  ███  ███  ███ ███         ".to_string(),
            "           ███   ███     ███  ███  ███       ███  ███████   █████      ".to_string(),
            "   ███     ███   ███ ███ ███  ███  ███       ███  ███          ███     ".to_string(),
            "    █████████     █████   ██████  █████     █████  ██████  ██████      ".to_string(),
        ]
    }

    pub fn fos_color(&self) -> Vec<String> {
        vec![
            "AAAAAAAAAAA                                                     AAAAAA ".to_string(),
            " BBB      B                                                    BBB  BBB".to_string(),
            " CCC   C     CCCCCC  CCCCCCCC   CCCCCCC  CCCCCC      CCCCCC    CCC     ".to_string(),
            " DDDDDDD    DDD  DDD  DDD  DDD DDD  DDD DDD  DDD    DDD  DDD DDDDDDD   ".to_string(),
            " EEE   E    EEE  EEE  EEE      EEE  EEE EEEEEEE     EEE  EEE   EEE     ".to_string(),
            " FFF        FFF  FFF  FFF      FFF  FFF FFF         FFF  FFF   FFF     ".to_string(),
            "GGGGG        GGGGGG  GGGGG      GGGGGGG  GGGGGG      GGGGGG   GGGGG    ".to_string(),
            "                                    HHH                                ".to_string(),
            "                               III  III                                ".to_string(),
            "    JJJJJJJJJ    JJJ            JJJJJJ       JJJ                       ".to_string(),
            "   KKK     KKK   KKK                                                   ".to_string(),
            "   LLL         LLLLLLL    LLLLLL  LLLLLLLL  LLLL   LLLLLL   LLLLL      ".to_string(),
            "    MMMMMMMMM    MMM     MMM  MMM  MMM  MMM  MMM  MMM  MMM MMM         ".to_string(),
            "           NNN   NNN     NNN  NNN  NNN       NNN  NNNNNNN   NNNNN      ".to_string(),
            "   OOO     OOO   OOO OOO OOO  OOO  OOO       OOO  OOO          OOO     ".to_string(),
            "    PPPPPPPPP     PPPPP   PPPPPP  PPPPP     PPPPP  PPPPPP  PPPPPP      ".to_string(),
        ]
    }
    pub fn fos_colormap(&self) -> HashMap<char, Color> {
        let mut color_map = HashMap::new();
        color_map.insert('A', Color::Rgb(255, 246, 161));
        color_map.insert('B', Color::Rgb(255, 235, 151));
        color_map.insert('C', Color::Rgb(255, 225, 141));
        color_map.insert('D', Color::Rgb(255, 208, 127));
        color_map.insert('E', Color::Rgb(255, 201, 121));
        color_map.insert('F', Color::Rgb(255, 193, 113));
        color_map.insert('G', Color::Rgb(255, 185, 106));
        color_map.insert('H', Color::Rgb(255, 176, 98));
        color_map.insert('I', Color::Rgb(255, 164, 88));
        color_map.insert('J', Color::Rgb(255, 154, 79));
        color_map.insert('K', Color::Rgb(255, 145, 72));
        color_map.insert('L', Color::Rgb(255, 134, 62));
        color_map.insert('M', Color::Rgb(255, 119, 48));
        color_map.insert('N', Color::Rgb(255, 109, 39));
        color_map.insert('O', Color::Rgb(255, 99, 30));
        color_map.insert('P', Color::Rgb(255, 85, 18));
        color_map
    }
}

impl Component for Logo {
    fn name(&self) -> &str {
        "logo"
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

    fn handle_action(&mut self, _action: &Action) -> ActionOutcome {
        ActionOutcome::NotHandled
    }

    fn render(&self, f: &mut ratatui::Frame, area: Rect) {
        let logo_lines: Vec<String> = if self.label == "Wizard" {
            self.wizard_lines()
        } else if self.label == "Forge of Stories" {
            self.fos_lines()
        } else {
            self.fos_lines()
        };
        let logo_color: Vec<String> = if self.label == "Wizard" {
            self.wizard_color()
        } else if self.label == "Forge of Stories" {
            self.fos_color()
        } else {
            self.fos_color()
        };

        let color_map: HashMap<char, Color> = if self.label == "Wizard" {
            self.wizard_colormap()
        } else if self.label == "Forge of Stories" {
            self.fos_colormap()
        } else {
            self.fos_colormap()
        };

        let mut styled_lines = Vec::new();

        for (_, (logo_line, color_line)) in logo_lines.iter().zip(logo_color.iter()).enumerate() {
            let mut spans = Vec::new();
            let logo_chars: Vec<char> = logo_line.chars().collect();
            let color_chars: Vec<char> = color_line.chars().collect();

            for (j, &logo_char) in logo_chars.iter().enumerate() {
                let color = if j < color_chars.len() {
                    color_map
                        .get(&color_chars[j])
                        .copied()
                        .unwrap_or(Color::White)
                } else {
                    Color::White
                };

                spans.push(Span::styled(
                    logo_char.to_string(),
                    Style::default().fg(color),
                ));
            }

            styled_lines.push(Line::from(spans));
        }
        let logo = Paragraph::new(styled_lines)
            .block(Block::default())
            .wrap(ratatui::widgets::Wrap { trim: false });
        // frame.render_widget(Block::new().style(Style::default().bg(Color::Green)), area);
        f.render_widget(logo, area);
    }
}

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

    fn handle_action(&mut self, _action: &Action) -> ActionOutcome {
        ActionOutcome::NotHandled
    }

    fn render(&self, f: &mut ratatui::Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(area);

        let title = Paragraph::new(Line::from(vec![Span::styled(
            self.title.clone(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Left);

        f.render_widget(title, layout[0]);

        if !self.body.is_empty() {
            let lines: Vec<Line> = self
                .body
                .iter()
                .map(|line| Line::from(Span::raw(line.clone())))
                .collect();
            let body = Paragraph::new(lines).wrap(Wrap { trim: false });
            f.render_widget(body, layout[1]);
        }
    }
}
