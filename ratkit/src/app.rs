extern crate alloc;

use alloc::format;
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

const DOT_MS: u64 = 60;
const DASH_MS: u64 = 180;
const STEP_THRESHOLD_SQ: f32 = 1.3 * 1.3;
const STEP_DEBOUNCE_MS: u64 = 300;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Morse,
    Imu,
}

pub struct App {
    mode: Mode,
    current_symbol: Vec<char>,
    output: Vec<char>,
    button_down: bool,
    menu_button_down: bool,
    last_press_ms: u64,
    last_release_ms: u64,
    clear_on_next_letter: bool,
    buzzer_on: bool,
    buzzer_until_ms: u64,
    ax: f32,
    ay: f32,
    az: f32,
    steps: u32,
    last_peak_ms: u64,
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: Mode::Morse,
            current_symbol: Vec::new(),
            output: Vec::new(),
            button_down: false,
            menu_button_down: false,
            last_press_ms: 0,
            last_release_ms: 0,
            clear_on_next_letter: false,
            buzzer_on: false,
            buzzer_until_ms: 0,
            ax: 0.0,
            ay: 0.0,
            az: 0.0,
            steps: 0,
            last_peak_ms: 0,
        }
    }

    pub fn tick(
        &mut self,
        now_ms: u64,
        button_pressed: bool,
        menu_pressed: bool,
        accel: Option<(f32, f32, f32)>,
    ) {
        if menu_pressed && !self.menu_button_down {
            self.menu_button_down = true;
            self.toggle_mode();
        } else if !menu_pressed {
            self.menu_button_down = false;
        }

        if let Some((ax, ay, az)) = accel {
            self.ax = ax;
            self.ay = ay;
            self.az = az;
            if self.mode == Mode::Imu {
                self.update_steps(now_ms);
            }
        }

        if self.mode != Mode::Morse {
            self.buzzer_on = false;
            self.button_down = false;
            return;
        }

        // Turn the buzzer off when its time is done.
        if self.buzzer_on && now_ms >= self.buzzer_until_ms {
            self.buzzer_on = false;
        }

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
                self.start_beep(now_ms, DOT_MS);
            } else {
                self.current_symbol.push('-');
                self.start_beep(now_ms, DASH_MS);
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
        match self.mode {
            Mode::Morse => {
                let mut lines = Vec::new();
                lines.push(Line::styled(
                    "Hold for dot / dash",
                    Style::default().fg(Color::Yellow),
                ).alignment(Alignment::Center));
                lines.push(Line::styled(
                    self.current_symbol.iter().collect::<String>(),
                    Style::default().fg(Color::Cyan),
                ));
                lines.push(
                    Line::styled(
                        self.output.iter().collect::<String>(),
                        Style::default().fg(Color::Green),
                    )
                    .alignment(Alignment::Center),
                );
                f.render_widget(
                    Paragraph::new(lines).block(
                        Block::bordered()
                            .border_style(Style::default().fg(Color::Yellow))
                            .border_type(BorderType::Rounded)
                            .title("┤ Rat Morse ├".white())
                            .title_alignment(Alignment::Center),
                    ),
                    f.area(),
                );
            }
            Mode::Imu => {
                let mut lines = Vec::new();
                lines.push(Line::styled(
                    "Powered MPU-6050",
                    Style::default().fg(Color::Cyan).bold(),
                ));
                lines.push(Line::raw(format!(
                    "Ax: {:>5.2}  Ay: {:>5.2}",
                    self.ax, self.ay
                )));
                lines.push(Line::raw(format!("Az: {:>5.2}", self.az)));
                lines.push(Line::raw(format!("Steps: {}", self.steps)));
                f.render_widget(
                    Paragraph::new(lines).block(
                        Block::bordered()
                            .border_style(Style::default().fg(Color::Yellow))
                            .border_type(BorderType::Rounded)
                            .title("┤ Rat Motion ├".white())
                            .title_alignment(Alignment::Center),
                    ),
                    f.area(),
                );
            }
        }
    }

    pub fn buzzer_on(&self) -> bool {
        self.buzzer_on
    }

    fn start_beep(&mut self, now_ms: u64, duration_ms: u64) {
        self.buzzer_on = true;
        self.buzzer_until_ms = now_ms + duration_ms;
    }

    fn toggle_mode(&mut self) {
        self.mode = if self.mode == Mode::Morse {
            Mode::Imu
        } else {
            Mode::Morse
        };
        self.buzzer_on = false;
    }

    fn update_steps(&mut self, now_ms: u64) {
        let mag_sq = self.ax * self.ax + self.ay * self.ay + self.az * self.az;
        if mag_sq > STEP_THRESHOLD_SQ && now_ms.saturating_sub(self.last_peak_ms) > STEP_DEBOUNCE_MS
        {
            self.steps = self.steps.saturating_add(1);
            self.last_peak_ms = now_ms;
        }
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
