use super::Component;
use crate::{action::Action, config::Config, utils::centered_rect};
use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Default)]
pub struct LogoComponent {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
}

impl LogoComponent {
    pub fn new() -> Self {
        Self::default()
    }
    pub(crate) fn length() -> u16 {
        71
    }
}

impl Component for LogoComponent {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                // add any logic here that should run on every tick
            }
            Action::Render => {
                // add any logic here that should run on every render
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, body: Rect) -> Result<()> {
        let area = centered_rect(90, 90, body);
        let logo_lines = vec![
            "███████████                                                     ██████",
            " ███      █                                                    ███  ███",
            " ███   █     ██████  ████████   ███████  ██████      ██████    ███     ",
            " ███████    ███  ███  ███  ███ ███  ███ ███  ███    ███  ███ ███████   ",
            " ███   █    ███  ███  ███      ███  ███ ███████     ███  ███   ███     ",
            " ███        ███  ███  ███      ███  ███ ███         ███  ███   ███     ",
            "█████        ██████  █████      ███████  ██████      ██████   █████    ",
            "                                    ███                                ",
            "                               ███  ███                                ",
            "    █████████    ███            ██████       ███                       ",
            "   ███     ███   ███                                                   ",
            "   ███         ███████    ██████  ████████  ████   ██████   █████      ",
            "    █████████    ███     ███  ███  ███  ███  ███  ███  ███ ███         ",
            "           ███   ███     ███  ███  ███       ███  ███████   █████      ",
            "   ███     ███   ███ ███ ███  ███  ███       ███  ███          ███     ",
            "    █████████     █████   ██████  █████     █████  ██████  ██████      ",
        ];

        let logo_color = vec![
            "AAAAABAAAAA                                                     AAAABA",
            " AAA      B                                                    AAB  AAA",
            " BAA   B     BBBABA  BAABABBA   ABABBBB  BBABAA      BBBBAB    ABB     ",
            " BBBBBAB    BBB  AAB  ABA  BBB BBA  BBB BBB  BBA    AAB  BBA BBBBBBB   ",
            " BBB   B    BBB  BBB  BBB      BBB  BBB BBBBBBB     BAB  BBB   BBB     ",
            " BBC        BBB  BBB  BBB      BBB  BBB BBB         BBB  BBB   BBC     ",
            "BBCCB        BBBBBB  BBBBC      BBCBBCB  BBCBCC      BBBBBB   CCBBC    ",
            "                                    CCC                                ",
            "                               CCC  CCC                                ",
            "    CCDDCCCDD    CDC            CCDDCC       CCD                       ",
            "   DDD     DDD   DDD                                                   ",
            "   DDD         DDDDDDD    DDDDDD  DDDDDDDD  DDDD   DDDDDD   DDDDD      ",
            "    DEDDEDDDE    EDD     DDD  DDD  DDD  DDD  DDD  DDD  DDE EEE         ",
            "           EEE   EED     EEE  DEE  DED       EDE  EEEEEEE   EEEEE      ",
            "   EEE     EEE   EEE EEE EEE  EEE  EEE       EEE  EEE          EEE     ",
            "    EEEEEEEEE     EEEEE   EEEEEE  EEEEE     EEEEE  EEEEEE  EEEEEE      ",
        ];

        let color_map: HashMap<char, Color> = [
            ('A', Color::Rgb(255, 246, 161)),
            ('B', Color::Rgb(255, 213, 101)),
            ('C', Color::Rgb(255, 169, 48)),
            ('D', Color::Rgb(255, 119, 8)),
            ('E', Color::Rgb(255, 85, 18)),
        ]
        .iter()
        .cloned()
        .collect();

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

        let logo_width: u16 = logo_lines
            .iter()
            .map(|s| s.chars().count() as u16)
            .max()
            .unwrap_or(0);
        let logo_height: u16 = logo_lines.len() as u16;

        let area = centered_rect(logo_width, logo_height, body);
        let logo = Paragraph::new(styled_lines)
            .block(Block::default())
            .wrap(ratatui::widgets::Wrap { trim: false });

        frame.render_widget(logo, area);
        Ok(())
    }
}
