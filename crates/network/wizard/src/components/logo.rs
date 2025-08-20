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
            "AAAAAAAAAAA                                                     AAAAAA",
            " BBB      B                                                    BBB  BBB",
            " CCC   C     CCCCCC  CCCCCCCC   CCCCCCC  CCCCCC      CCCCCC    CCC     ",
            " DDDDDDD    DDD  DDD  DDD  DDD DDD  DDD DDD  DDD    DDD  DDD DDDDDDD   ",
            " EEE   E    EEE  EEE  EEE      EEE  EEE EEEEEEE     EEE  EEE   EEE     ",
            " FFF        FFF  FFF  FFF      FFF  FFF FFF         FFF  FFF   FFF     ",
            "GGGGG        GGGGGG  GGGGG      GGGGGGG  GGGGGG      GGGGGG   GGGGG    ",
            "                                    HHH                                ",
            "                               III  III                                ",
            "    JJJJJJJJJ    JJJ            JJJJJJ       JJJ                       ",
            "   KKK     KKK   KKK                                                   ",
            "   LLL         LLLLLLL    LLLLLL  LLLLLLLL  LLLL   LLLLLL   LLLLL      ",
            "    MMMMMMMMM    MMM     MMM  MMM  MMM  MMM  MMM  MMM  MMM MMM         ",
            "           NNN   NNN     NNN  NNN  NNN       NNN  NNNNNNN   NNNNN      ",
            "   OOO     OOO   OOO OOO OOO  OOO  OOO       OOO  OOO          OOO     ",
            "    PPPPPPPPP     PPPPP   PPPPPP  PPPPP     PPPPP  PPPPPP  PPPPPP      ",
        ];

        let color_map: HashMap<char, Color> = [
            ('A', Color::Rgb(255, 246, 161)),
            ('B', Color::Rgb(255, 235, 151)),
            ('C', Color::Rgb(255, 225, 141)),
            ('D', Color::Rgb(255, 208, 127)),
            ('E', Color::Rgb(255, 201, 121)),
            ('F', Color::Rgb(255, 193, 113)),
            ('G', Color::Rgb(255, 185, 106)),
            ('H', Color::Rgb(255, 176, 98)),
            ('I', Color::Rgb(255, 164, 88)),
            ('J', Color::Rgb(255, 154, 79)),
            ('K', Color::Rgb(255, 145, 72)),
            ('L', Color::Rgb(255, 134, 62)),
            ('M', Color::Rgb(255, 119, 48)),
            ('N', Color::Rgb(255, 109, 39)),
            ('O', Color::Rgb(255, 99, 30)),
            ('P', Color::Rgb(255, 85, 18)),
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
