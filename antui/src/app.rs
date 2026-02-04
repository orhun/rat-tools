extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use ratatui::text::Line;
use ratatui::{
    layout::Alignment,
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Paragraph},
};

const DOT_DASH_MS: u64 = 200;
const LETTER_GAP_MS: u64 = 500;
const WORD_GAP_MS: u64 = 1200;

pub struct App {
    current_symbol: Vec<char>,
    output: Vec<char>,
    button_down: bool,
    last_press_ms: u64,
    last_release_ms: u64,
    clear_on_next_letter: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            current_symbol: Vec::new(),
            output: Vec::new(),
            button_down: false,
            last_press_ms: 0,
            last_release_ms: 0,
            clear_on_next_letter: false,
        }
    }

    pub fn tick(&mut self, now_ms: u64, button_pressed: bool) {
        // Detect transitions.
        if button_pressed && !self.button_down {
            self.button_down = true;
            self.last_press_ms = now_ms;
            return;
        }

        if !button_pressed && self.button_down {
            self.button_down = false;
            let press_ms = now_ms.saturating_sub(self.last_press_ms);
            if press_ms < DOT_DASH_MS {
                self.current_symbol.push('.');
            } else {
                self.current_symbol.push('-');
            }
            self.last_release_ms = now_ms;
            return;
        }

        // Gap detection (button is up).
        if !self.button_down {
            let gap_ms = now_ms.saturating_sub(self.last_release_ms);

            if !self.current_symbol.is_empty() && gap_ms >= LETTER_GAP_MS {
                let letter = decode_morse(&self.current_symbol);
                if self.clear_on_next_letter {
                    self.output.clear();
                    self.clear_on_next_letter = false;
                }
                self.output.push(letter);
                self.current_symbol.clear();
            }

            if self.current_symbol.is_empty() && gap_ms >= WORD_GAP_MS {
                // Mark that the next letter starts a new word.
                if !self.output.is_empty() {
                    self.clear_on_next_letter = true;
                }
            }
        }
    }

    pub fn render(&self, f: &mut ratatui::Frame) {
        let mut lines = Vec::new();
        lines.push(Line::styled(
            self.current_symbol.iter().collect::<String>(),
            Style::default().fg(Color::Cyan),
        ));
        lines.push(Line::styled(
            self.output.iter().collect::<String>(),
            Style::default().fg(Color::Green),
        ));
        f.render_widget(
            Paragraph::new(lines).block(
                Block::bordered()
                    .border_style(Style::default().fg(Color::Yellow))
                    .border_type(BorderType::Rounded)
                    .title("Rat Morse".white())
                    .title_alignment(Alignment::Center),
            ),
            f.area(),
        );
    }
}

fn decode_morse(symbol: &[char]) -> char {
    let mut s = String::new();
    for c in symbol {
        s.push(*c);
    }

    for (code, letter) in MORSE_TABLE {
        if *code == s {
            return *letter;
        }
    }
    '?'
}

const MORSE_TABLE: &[(&str, char)] = &[
    (".-", 'A'),
    ("-...", 'B'),
    ("-.-.", 'C'),
    ("-..", 'D'),
    (".", 'E'),
    ("..-.", 'F'),
    ("--.", 'G'),
    ("....", 'H'),
    ("..", 'I'),
    (".---", 'J'),
    ("-.-", 'K'),
    (".-..", 'L'),
    ("--", 'M'),
    ("-.", 'N'),
    ("---", 'O'),
    (".--.", 'P'),
    ("--.-", 'Q'),
    (".-.", 'R'),
    ("...", 'S'),
    ("-", 'T'),
    ("..-", 'U'),
    ("...-", 'V'),
    (".--", 'W'),
    ("-..-", 'X'),
    ("-.--", 'Y'),
    ("--..", 'Z'),
];
