extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::{format, vec};

use ratatui::layout::Margin;
use ratatui::text::Line;
use ratatui::{
    layout::Alignment,
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Paragraph},
};
use tui_big_text::{BigText, PixelSize};

const DOT_DASH_MS: u64 = 200;
const LETTER_GAP_MS: u64 = 500;
const WORD_GAP_MS: u64 = 1200;

const DOT_MS: u64 = 60;
const DASH_MS: u64 = 180;
const STEP_THRESHOLD_SQ: f32 = 1.3 * 1.3;
const STEP_DEBOUNCE_MS: u64 = 300;
const INTRO_BEEP_COUNT: u8 = 3;
const INTRO_BEEP_MS: u64 = 80;
const INTRO_BEEP_GAP_MS: u64 = 120;
const TRANSITION_BEEP_MS: u64 = 120;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Game,
    Imu,
    Morse,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Intro,
    Walk,
    Stop,
    Morse,
    Success,
    Complete,
}

const LEVELS: &[(&str, u32)] = &[("rat", 5), ("tui", 10), ("cheese", 15)];

pub struct App {
    mode: Mode,
    stage: Stage,
    level: usize,
    target_steps: u32,
    start_steps: u32,
    current_word: &'static str,
    stop_time_ms: u64,
    success_time_ms: u64,
    intro_time_ms: u64,
    intro_elapsed_ms: u64,
    rng_state: u32,
    walk_hint: u8,
    intro_beeps_done: u8,
    intro_next_beep_ms: u64,
    transition_beep_pending: bool,
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
            mode: Mode::Game,
            stage: Stage::Intro,
            level: 0,
            target_steps: 0,
            start_steps: 0,
            current_word: LEVELS[0].0,
            stop_time_ms: 0,
            success_time_ms: 0,
            intro_time_ms: 0,
            intro_elapsed_ms: 0,
            rng_state: 0,
            walk_hint: 0,
            intro_beeps_done: 0,
            intro_next_beep_ms: 0,
            transition_beep_pending: false,
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
            if self.mode == Mode::Imu || self.stage == Stage::Walk {
                self.update_steps(now_ms);
            }
        }

        if self.mode == Mode::Imu {
            self.buzzer_on = false;
            self.button_down = false;
            return;
        }

        // Turn the buzzer off when its time is done.
        if self.buzzer_on && now_ms >= self.buzzer_until_ms {
            self.buzzer_on = false;
        }

        if self.mode == Mode::Game && self.stage != Stage::Morse {
            self.button_down = false;
            self.update_stage_beeps(now_ms);
            self.update_stage(now_ms);
            return;
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

        if self.mode == Mode::Game {
            self.update_stage(now_ms);
        }
    }

    pub fn render(&self, f: &mut ratatui::Frame) {
        match self.mode {
            Mode::Game => {
                self.render_game(f);
            }
            Mode::Imu => {
                let mut lines = Vec::new();
                lines.push(Line::styled(
                    "Powered by MPU-6050",
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
                            .title("┤ Sensor Values ├".white())
                            .title_alignment(Alignment::Center),
                    ),
                    f.area(),
                );
            }
            Mode::Morse => {
                self.render_morse_play(f);
            }
        }
    }

    #[allow(dead_code)]
    pub fn buzzer_on(&self) -> bool {
        self.buzzer_on
    }

    fn start_beep(&mut self, now_ms: u64, duration_ms: u64) {
        self.buzzer_on = true;
        self.buzzer_until_ms = now_ms + duration_ms;
    }

    fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            Mode::Game => Mode::Imu,
            Mode::Imu => Mode::Morse,
            Mode::Morse => Mode::Game,
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

    fn update_stage(&mut self, now_ms: u64) {
        match self.stage {
            Stage::Intro => {
                if self.intro_time_ms == 0 {
                    self.intro_time_ms = now_ms;
                    self.intro_beeps_done = 0;
                    self.intro_next_beep_ms = now_ms;
                }
                self.intro_elapsed_ms = now_ms.saturating_sub(self.intro_time_ms);
                if now_ms.saturating_sub(self.intro_time_ms) >= 3000 {
                    self.setup_level();
                    self.stage = Stage::Walk;
                }
            }
            Stage::Walk => {
                if self.steps.saturating_sub(self.start_steps) >= self.target_steps {
                    self.stage = Stage::Stop;
                    self.stop_time_ms = now_ms;
                }
            }
            Stage::Stop => {
                if now_ms.saturating_sub(self.stop_time_ms) > 1500 {
                    self.stage = Stage::Morse;
                    self.clear_morse();
                }
            }
            Stage::Morse => {
                if self.morse_complete() && self.check_word() {
                    self.stage = Stage::Success;
                    self.success_time_ms = now_ms;
                }
            }
            Stage::Success => {
                if now_ms.saturating_sub(self.success_time_ms) > 2000 {
                    self.next_level();
                }
            }
            Stage::Complete => {}
        }
    }

    fn render_game(&self, frame: &mut ratatui::Frame) {
        let mut lines = Vec::new();
        match self.stage {
            Stage::Intro => {
                let title = "$ CHEESE LOCATOR >>>>";
                let total_ms = 2000u64;
                let len = title.len() as u64;
                let shown = if len == 0 {
                    0
                } else {
                    let count = (self.intro_elapsed_ms.saturating_mul(len)) / total_ms;
                    count.min(len) as usize
                };
                let reveal = &title[..shown];
                lines.push(Line::default());
                lines.push(Line::default());
                lines.push(Line::default());
                lines.push(Line::raw(reveal));
                frame.render_widget(Paragraph::new(lines), frame.area());
                return;
            }
            Stage::Walk => {
                let walked = self.steps.saturating_sub(self.start_steps);
                let left = self.target_steps.saturating_sub(walked);
                let (label, arrows) = match self.walk_hint {
                    1 => ("It's on the LEFT", "←←←"),
                    2 => ("It's on the RIGHT", "→→→"),
                    _ => ("It's STRAIGHT AHEAD", "↑↑↑"),
                };
                lines.push(Line::default());
                lines.push(
                    Line::styled(
                        format!("{}!", label),
                        Style::default().fg(Color::Green).bold(),
                    )
                    .alignment(Alignment::Center),
                );
                lines.push(
                    Line::styled(arrows, Style::default().fg(Color::Yellow))
                        .alignment(Alignment::Center),
                );
                lines.push(
                    Line::styled(format!("Steps left: {}", left), Style::default().bold())
                        .alignment(Alignment::Center),
                );
            }
            Stage::Stop => {
                lines.push(Line::default());
                lines.push(
                    Line::styled("Stop!", Style::default().fg(Color::Yellow).bold())
                        .alignment(Alignment::Center),
                );
                lines.push(Line::raw("Hold still...").alignment(Alignment::Center));
            }
            Stage::Morse => {
                lines.push(
                    Line::styled(
                        format!(
                            "Spell: {}",
                            self.current_word
                                .chars()
                                .map(|c| format!("{c} ").to_uppercase())
                                .collect::<String>()
                        ),
                        Style::default().fg(Color::Yellow),
                    )
                    .alignment(Alignment::Center),
                );
                lines.push(
                    Line::styled("Hold for dot / dash", Style::default().fg(Color::Cyan))
                        .alignment(Alignment::Center),
                );
                lines.push(
                    Line::styled(
                        self.current_symbol.iter().collect::<String>(),
                        Style::default().fg(Color::Cyan),
                    )
                    .alignment(Alignment::Center),
                );
                lines.push(
                    Line::styled(
                        self.output
                            .iter()
                            .map(|c| format!("{c} ").to_uppercase())
                            .collect::<String>(),
                        Style::default().fg(Color::Green),
                    )
                    .alignment(Alignment::Center),
                );
            }
            Stage::Success => {
                lines.push(
                    Line::styled("congRATs!", Style::default().fg(Color::Green).bold())
                        .alignment(Alignment::Center),
                );
                lines.push(Line::raw("Next level...").alignment(Alignment::Center));
            }
            Stage::Complete => {
                let big_text = BigText::builder()
                    .pixel_size(PixelSize::Quadrant)
                    .style(Style::new().blue())
                    .lines(vec!["CHEZ!!".yellow().into()])
                    .build();
                frame.render_widget(
                    big_text,
                    frame.area().inner(Margin {
                        horizontal: 1,
                        vertical: 1,
                    }),
                );
                return;
            }
        }

        frame.render_widget(
            Paragraph::new(lines).block(
                Block::bordered()
                    .border_style(Style::default().fg(Color::Yellow))
                    .border_type(BorderType::Rounded)
                    .title(format!("┤ Level: {} ├", self.level + 1).white())
                    .title_alignment(Alignment::Center),
            ),
            frame.area(),
        );
    }

    fn render_morse_play(&self, frame: &mut ratatui::Frame) {
        let mut lines = Vec::new();
        lines.push(
            Line::styled("Hold for dot / dash", Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center),
        );
        lines.push(
            Line::styled(
                self.current_symbol.iter().collect::<String>(),
                Style::default().fg(Color::Cyan),
            )
            .alignment(Alignment::Center),
        );

        lines.push(
            Line::styled(
                self.output.iter().collect::<String>(),
                Style::default().fg(Color::Green),
            )
            .alignment(Alignment::Center),
        );
        frame.render_widget(
            Paragraph::new(lines).block(
                Block::bordered()
                    .border_style(Style::default().fg(Color::Yellow))
                    .border_type(BorderType::Rounded)
                    .title("┤ Morse Decoder ├".white())
                    .title_alignment(Alignment::Center),
            ),
            frame.area(),
        );
    }

    fn setup_level(&mut self) {
        let (word, steps) = LEVELS[self.level];
        self.current_word = word;
        if self.rng_state == 0 {
            self.rng_state = (self.intro_time_ms as u32)
                .wrapping_mul(1664525)
                .wrapping_add(1013904223);
        }
        let bonus = 5 + (self.next_u32() % 6); // 5..=10
        self.target_steps = steps + bonus;
        self.start_steps = self.steps;
        self.walk_hint = (self.next_u32() % 3) as u8;
    }

    fn clear_morse(&mut self) {
        self.output.clear();
        self.current_symbol.clear();
        self.clear_on_next_letter = false;
        self.last_release_ms = 0;
    }

    fn morse_complete(&self) -> bool {
        if !self.current_symbol.is_empty() {
            return false;
        }
        if self.output.is_empty() {
            return false;
        }
        self.output.len() == self.current_word.len()
    }

    fn check_word(&self) -> bool {
        if self.output.len() != self.current_word.len() {
            return false;
        }
        for (i, ch) in self.output.iter().enumerate() {
            let expected = self.current_word.as_bytes()[i] as char;
            if ch.to_ascii_lowercase() != expected {
                return false;
            }
        }
        true
    }

    fn next_level(&mut self) {
        self.level = self.level.saturating_add(1);
        if self.level >= LEVELS.len() {
            self.stage = Stage::Complete;
        } else {
            self.stage = Stage::Intro;
            self.intro_time_ms = 0;
            self.intro_elapsed_ms = 0;
            self.intro_beeps_done = 0;
            self.intro_next_beep_ms = 0;
            self.transition_beep_pending = true;
        }
    }

    fn next_u32(&mut self) -> u32 {
        // Simple LCG for gameplay randomness.
        self.rng_state = self
            .rng_state
            .wrapping_mul(1664525)
            .wrapping_add(1013904223);
        self.rng_state
    }

    fn update_stage_beeps(&mut self, now_ms: u64) {
        if self.stage == Stage::Complete {
            self.buzzer_on = true;
            self.buzzer_until_ms = u64::MAX;
            return;
        }

        if self.transition_beep_pending && !self.buzzer_on {
            self.start_beep(now_ms, TRANSITION_BEEP_MS);
            self.transition_beep_pending = false;
            return;
        }

        if self.stage == Stage::Intro && self.intro_beeps_done < INTRO_BEEP_COUNT {
            if now_ms >= self.intro_next_beep_ms {
                self.start_beep(now_ms, INTRO_BEEP_MS);
                self.intro_beeps_done = self.intro_beeps_done.saturating_add(1);
                self.intro_next_beep_ms = now_ms + INTRO_BEEP_MS + INTRO_BEEP_GAP_MS;
            }
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
