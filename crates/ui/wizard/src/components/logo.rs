use super::Component;
use crate::{action::Action, tui::Frame};
use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Default)]
pub struct LogoComponent {
    command_tx: Option<UnboundedSender<Action>>,
}

impl LogoComponent {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for LogoComponent {
    fn height_constraint(&self) -> Constraint {
        Constraint::Max(1)
    }

    fn name(&self) -> &'static str {
        "logo"
    }

    fn draw(&mut self, frame: &mut Frame, body: Rect) -> Result<()> {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Max(16), Constraint::Min(0)])
            .split(body);
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Max(71), Constraint::Min(0)])
            .split(vertical[1]);

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
                    // color_map
                    //     .get(&color_chars[j])
                    //     .copied()
                    //     .unwrap_or(Color::White)
                    Color::DarkGray
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

        frame.render_widget(logo, horizontal[1]);
        Ok(())
    }
}

#[derive(Default)]
pub struct WizardLogoComponent {
    command_tx: Option<UnboundedSender<Action>>,
}
impl WizardLogoComponent {
    pub fn new() -> Self {
        Self::default()
    }
    pub(crate) fn length() -> u16 {
        49
    }
}
impl Component for WizardLogoComponent {
    fn height_constraint(&self) -> Constraint {
        Constraint::Max(1)
    }

    fn name(&self) -> &'static str {
        "wizard_logo"
    }

    fn draw(&mut self, frame: &mut Frame, body: Rect) -> Result<()> {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Max(16), Constraint::Min(0)])
            .split(body);
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Max(71), Constraint::Min(0)])
            .split(vertical[1]);

        let logo_lines = vec![
            "██        ██ ██ ███████  █████  ██████  ██████",
            "██   ██   ██ ██      ██ ██   ██ ██   ██ ██   ██",
            " ██ ████ ██  ██   ███   ███████ ██████  ██   ██",
            " ████  ████  ██ ██      ██   ██ ██   ██ ██   ██",
            "  ██    ██   ██ ███████ ██   ██ ██   ██ ██████",
        ];

        let logo_color = vec![
            "AA        DD EE EFFFGGG  HHIII  JJKKKL  LMMMNN",
            "AA   BC   DD EE      GG HH   IJ JJ   LL LM   NN",
            " AA BBCC DD  EE   FFG   HHHIIIJ JJKKKL  LM   NN",
            " AABB  CCDD  EE EF      HH   IJ JJ   LL LM   NN",
            "  AB    CD   EE EFFFGGG HH   IJ JJ   LL LMMMNN",
        ];

        let color_map: HashMap<char, Color> = [
            ('A', Color::Rgb(91, 0, 130)),
            ('B', Color::Rgb(85, 1, 129)),
            ('C', Color::Rgb(68, 3, 127)),
            ('D', Color::Rgb(54, 4, 126)),
            ('E', Color::Rgb(38, 6, 124)),
            ('F', Color::Rgb(30, 7, 123)),
            ('G', Color::Rgb(20, 8, 122)),
            ('H', Color::Rgb(13, 9, 121)),
            ('I', Color::Rgb(8, 17, 129)),
            ('J', Color::Rgb(7, 38, 151)),
            ('K', Color::Rgb(5, 64, 178)),
            ('L', Color::Rgb(3, 89, 203)),
            ('M', Color::Rgb(1, 116, 230)),
            ('N', Color::Rgb(0, 140, 255)),
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

        let logo = Paragraph::new(styled_lines)
            .block(Block::default())
            .wrap(ratatui::widgets::Wrap { trim: false });

        frame.render_widget(logo, horizontal[1]);
        Ok(())
    }
}
