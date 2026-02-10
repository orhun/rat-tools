use alloc::vec::Vec;

use alloc::vec;
use libm::{cos, sin};
use ratatui::prelude::*;
use ratatui::widgets::{Axis, Chart, Dataset, GraphType};

const SPIRO_POINTS: usize = 160;
const LACE_POINTS: usize = 140;
const SHARDS: usize = 5;
const Y_SCALE: f64 = 0.7;
const TICK_DIV: u32 = 2;

#[inline]
fn advance_sin_cos(sin_v: &mut f64, cos_v: &mut f64, sin_delta: f64, cos_delta: f64) {
    let next_sin = *sin_v * cos_delta + *cos_v * sin_delta;
    let next_cos = *cos_v * cos_delta - *sin_v * sin_delta;
    *sin_v = next_sin;
    *cos_v = next_cos;
}

pub struct HyperApp {
    spiro: Spiro,
    lace: Lace,
    shards: Vec<Shard>,
    bounds: f64,
    tick: u32,
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
        let steps = (self.points.len() - 1) as f64;
        let delta = core::f64::consts::TAU / steps;
        let angle0 = self.phase * 0.8;
        let angle2_factor = (self.r - self.r2) / self.r2;
        let angle2_0 = angle2_factor * angle0 + self.phase;
        let wobble0 = self.phase;
        let delta2 = angle2_factor * delta;
        let delta_wobble = 8.0 * delta;
        let (sin_d, cos_d) = (sin(delta), cos(delta));
        let (sin_d2, cos_d2) = (sin(delta2), cos(delta2));
        let (sin_dw, cos_dw) = (sin(delta_wobble), cos(delta_wobble));
        let (mut sin_a, mut cos_a) = (sin(angle0), cos(angle0));
        let (mut sin_a2, mut cos_a2) = (sin(angle2_0), cos(angle2_0));
        let (mut sin_w, mut cos_w) = (sin(wobble0), cos(wobble0));
        for point in &mut self.points {
            let x = (self.r - self.r2) * cos_a + self.d * cos_a2;
            let y = (self.r - self.r2) * sin_a - self.d * sin_a2;
            let wobble = sin_w * 0.6;
            *point = (
                (x + wobble) * 0.7,
                (y + wobble) * 0.7 * Y_SCALE * (1.0 + 0.2 * k),
            );
            advance_sin_cos(&mut sin_a, &mut cos_a, sin_d, cos_d);
            advance_sin_cos(&mut sin_a2, &mut cos_a2, sin_d2, cos_d2);
            advance_sin_cos(&mut sin_w, &mut cos_w, sin_dw, cos_dw);
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
        let steps = (self.points.len() - 1) as f64;
        let delta = core::f64::consts::TAU / steps;
        let (sin_da, cos_da) = (sin(3.0 * delta), cos(3.0 * delta));
        let (sin_db, cos_db) = (sin(7.0 * delta), cos(7.0 * delta));
        let (sin_ds, cos_ds) = (sin(2.0 * delta), cos(2.0 * delta));
        let (sin_dc, cos_dc) = (sin(1.7 * delta), cos(1.7 * delta));
        let (mut sin_a, mut cos_a) = (sin(self.phase * 1.4), cos(self.phase * 1.4));
        let (mut sin_b, mut cos_b) = (sin(-self.phase * 0.9), cos(-self.phase * 0.9));
        let (mut sin_s, mut cos_s) = (sin(self.phase), cos(self.phase));
        let (mut sin_c, mut cos_c) = (sin(self.phase), cos(self.phase));
        let mut t = 0.0;
        for point in &mut self.points {
            let r = 14.0 + sin_a * 4.0 + cos_b * 3.0;
            let x = cos(t + sin_s * 0.6) * r;
            let y = sin(t + cos_c * 0.6) * r * Y_SCALE;
            *point = (x, y);
            t += delta;
            advance_sin_cos(&mut sin_a, &mut cos_a, sin_da, cos_da);
            advance_sin_cos(&mut sin_b, &mut cos_b, sin_db, cos_db);
            advance_sin_cos(&mut sin_s, &mut cos_s, sin_ds, cos_ds);
            advance_sin_cos(&mut sin_c, &mut cos_c, sin_dc, cos_dc);
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
        let steps = (len - 1) as f64;
        let t_step = 1.0 / steps;
        let (sin_dr, cos_dr) = (sin(9.0 * t_step), cos(9.0 * t_step));
        let (sin_dc, cos_dc) = (sin(6.0 * t_step), cos(6.0 * t_step));
        let (mut sin_r, mut cos_r) = (sin(self.phase), cos(self.phase));
        let (mut sin_c, mut cos_c) = (sin(self.phase), cos(self.phase));
        let mut t = 0.0;
        for point in &mut self.points {
            let r = self.radius + sin_r * 2.2;
            let a = self.angle + (t - 0.5) * self.span + cos_c * 0.3;
            let x = cos(a) * r;
            let y = sin(a) * r * Y_SCALE;
            *point = (x, y);
            t += t_step;
            advance_sin_cos(&mut sin_r, &mut cos_r, sin_dr, cos_dr);
            advance_sin_cos(&mut sin_c, &mut cos_c, sin_dc, cos_dc);
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
            tick: 0,
        }
    }

    pub fn on_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        if self.tick % TICK_DIV != 0 {
            return;
        }
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
