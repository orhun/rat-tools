use alloc::vec::Vec;

use alloc::vec;
use libm::{cos, sin};
use ratatui::prelude::*;
use ratatui::widgets::{Axis, Chart, Dataset, GraphType};

const SPIRO_POINTS: usize = 320;
const LACE_POINTS: usize = 260;
const SHARDS: usize = 8;
const Y_SCALE: f64 = 0.7;

pub struct HyperApp {
    spiro: Spiro,
    lace: Lace,
    shards: Vec<Shard>,
    bounds: f64,
}

struct Spiro {
    points: Vec<(f64, f64)>,
    phase: f64,
    speed: f64,
    r: f64,
    r2: f64,
    d: f64,
    color: Color,
}

impl Spiro {
    fn new(speed: f64, r: f64, r2: f64, d: f64, color: Color) -> Self {
        let mut spiro = Self {
            points: vec![(0.0, 0.0); SPIRO_POINTS],
            phase: 0.0,
            speed,
            r,
            r2,
            d,
            color,
        };
        spiro.recompute();
        spiro
    }

    fn tick(&mut self) {
        self.phase += self.speed;
        self.recompute();
    }

    fn recompute(&mut self) {
        let k = self.r / self.r2;
        for (idx, point) in self.points.iter_mut().enumerate() {
            let t = idx as f64 / (SPIRO_POINTS - 1) as f64 * core::f64::consts::TAU;
            let angle = t + self.phase * 0.8;
            let x = (self.r - self.r2) * cos(angle) +
                self.d * cos(((self.r - self.r2) / self.r2) * angle + self.phase);
            let y = (self.r - self.r2) * sin(angle) -
                self.d * sin(((self.r - self.r2) / self.r2) * angle + self.phase);
            let wobble = sin(t * 8.0 + self.phase) * 0.6;
            *point = ((x + wobble) * 0.7, (y + wobble) * 0.7 * Y_SCALE * (1.0 + 0.2 * k));
        }
    }
}

struct Lace {
    points: Vec<(f64, f64)>,
    phase: f64,
    speed: f64,
    color: Color,
}

impl Lace {
    fn new(speed: f64, color: Color) -> Self {
        let mut lace = Self {
            points: vec![(0.0, 0.0); LACE_POINTS],
            phase: 0.0,
            speed,
            color,
        };
        lace.recompute();
        lace
    }

    fn tick(&mut self) {
        self.phase += self.speed;
        self.recompute();
    }

    fn recompute(&mut self) {
        for (idx, point) in self.points.iter_mut().enumerate() {
            let t = idx as f64 / (LACE_POINTS - 1) as f64 * core::f64::consts::TAU;
            let a = t * 3.0 + self.phase * 1.4;
            let b = t * 7.0 - self.phase * 0.9;
            let r = 14.0 + sin(a) * 4.0 + cos(b) * 3.0;
            let x = cos(t + sin(self.phase + t * 2.0) * 0.6) * r;
            let y = sin(t + cos(self.phase + t * 1.7) * 0.6) * r * Y_SCALE;
            *point = (x, y);
        }
    }
}

struct Shard {
    points: Vec<(f64, f64)>,
    phase: f64,
    speed: f64,
    angle: f64,
    span: f64,
    radius: f64,
    color: Color,
}

impl Shard {
    fn new(phase: f64, speed: f64, angle: f64, span: f64, radius: f64, color: Color) -> Self {
        let mut shard = Self {
            points: vec![(0.0, 0.0); SPIRO_POINTS / 2],
            phase,
            speed,
            angle,
            span,
            radius,
            color,
        };
        shard.recompute();
        shard
    }

    fn tick(&mut self) {
        self.phase += self.speed;
        self.recompute();
    }

    fn recompute(&mut self) {
        let len = self.points.len();
        for (idx, point) in self.points.iter_mut().enumerate() {
            let t = idx as f64 / (len - 1) as f64;
            let r = self.radius + sin(self.phase + t * 9.0) * 2.2;
            let a = self.angle + (t - 0.5) * self.span + cos(self.phase + t * 6.0) * 0.3;
            let x = cos(a) * r;
            let y = sin(a) * r * Y_SCALE;
            *point = (x, y);
        }
    }
}

impl HyperApp {
    pub fn new() -> Self {
        let spiro = Spiro::new(0.04, 12.0, 5.0, 9.0, Color::LightMagenta);
        let lace = Lace::new(0.05, Color::Blue);
        let mut shards = Vec::with_capacity(SHARDS);
        for i in 0..SHARDS {
            let angle = i as f64 / SHARDS as f64 * core::f64::consts::TAU;
            let speed = 0.03 + (i as f64) * 0.003;
            let span = 1.1 + (i as f64) * 0.05;
            let radius = 16.0 + (i as f64) * 0.6;
            let color = if i % 3 == 0 {
                Color::Magenta
            } else if i % 3 == 1 {
                Color::LightBlue
            } else {
                Color::Cyan
            };
            shards.push(Shard::new(angle * 0.5, speed, angle, span, radius, color));
        }
        let bounds = 24.0;

        Self {
            spiro,
            lace,
            shards,
            bounds,
        }
    }

    pub fn on_tick(&mut self) {
        self.spiro.tick();
        self.lace.tick();
        for shard in &mut self.shards {
            shard.tick();
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let mut datasets = Vec::with_capacity(2 + self.shards.len());

        datasets.push(
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(self.spiro.color))
                .data(&self.spiro.points),
        );

        datasets.push(
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(self.lace.color))
                .data(&self.lace.points),
        );

        for shard in &self.shards {
            datasets.push(
                Dataset::default()
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(shard.color))
                    .data(&shard.points),
            );
        }

        let chart = Chart::new(datasets)
            .x_axis(
                Axis::default()
                    .style(Style::default().fg(Color::Black))
                    .bounds([-self.bounds, self.bounds]),
            )
            .y_axis(
                Axis::default()
                    .style(Style::default().fg(Color::Black))
                    .bounds([-self.bounds * Y_SCALE, self.bounds * Y_SCALE]),
            );

        frame.render_widget(chart, frame.area());
    }
}
