use alloc::vec::Vec;

use alloc::vec;
use libm::{cos, sin};
use ratatui::prelude::*;
use ratatui::widgets::{Axis, Chart, Dataset, GraphType};

const FIELD_POINTS: usize = 120;
const RING_POINTS: usize = 96;
const STEP: f64 = 0.08;
const Y_SCALE: f64 = 0.6;
const TICK_DIV: u32 = 1;

#[inline]
fn advance_sin_cos(sin_v: &mut f64, cos_v: &mut f64, sin_delta: f64, cos_delta: f64) {
    let next_sin = *sin_v * cos_delta + *cos_v * sin_delta;
    let next_cos = *cos_v * cos_delta - *sin_v * sin_delta;
    *sin_v = next_sin;
    *cos_v = next_cos;
}

pub struct AuroraApp {
    fields: Vec<Field>,
    ring: Ring,
    bounds: f64,
    tick: u32,
}

struct Field {
    points: Vec<(f64, f64)>,
    freq_x: f64,
    freq_y: f64,
    phase: f64,
    speed: f64,
    scale: f64,
    color: Color,
}

impl Field {
    fn new(freq_x: f64, freq_y: f64, scale: f64, speed: f64, color: Color) -> Self {
        let mut field = Self {
            points: vec![(0.0, 0.0); FIELD_POINTS],
            freq_x,
            freq_y,
            phase: 0.0,
            speed,
            scale,
            color,
        };
        field.recompute();
        field
    }

    fn tick(&mut self) {
        self.phase += self.speed;
        self.recompute();
    }

    fn recompute(&mut self) {
        let delta_x = STEP * self.freq_x;
        let delta_y = STEP * self.freq_y;
        let (mut sin_x, mut cos_x) = (sin(self.phase), cos(self.phase));
        let (mut sin_y, mut cos_y) = (sin(self.phase * 1.3), cos(self.phase * 1.3));
        let (sin_dx, cos_dx) = (sin(delta_x), cos(delta_x));
        let (sin_dy, cos_dy) = (sin(delta_y), cos(delta_y));
        for point in &mut self.points {
            let x = cos_x * self.scale;
            let y = sin_y * self.scale * Y_SCALE;
            *point = (x, y);
            advance_sin_cos(&mut sin_x, &mut cos_x, sin_dx, cos_dx);
            advance_sin_cos(&mut sin_y, &mut cos_y, sin_dy, cos_dy);
        }
    }
}

struct Ring {
    points: Vec<(f64, f64)>,
    base_radius: f64,
    wobble: f64,
    phase: f64,
    speed: f64,
    color: Color,
}

impl Ring {
    fn new(base_radius: f64, wobble: f64, speed: f64, color: Color) -> Self {
        let mut ring = Self {
            points: vec![(0.0, 0.0); RING_POINTS],
            base_radius,
            wobble,
            phase: 0.0,
            speed,
            color,
        };
        ring.recompute();
        ring
    }

    fn tick(&mut self) {
        self.phase += self.speed;
        self.recompute();
    }

    fn recompute(&mut self) {
        let radius = self.base_radius + sin(self.phase) * self.wobble;
        let step = core::f64::consts::TAU / self.points.len() as f64;
        let (sin_step, cos_step) = (sin(step), cos(step));
        let (mut sin_a, mut cos_a) = (0.0_f64, 1.0_f64);
        for point in &mut self.points {
            let x = cos_a * radius;
            let y = sin_a * radius * Y_SCALE;
            *point = (x, y);
            advance_sin_cos(&mut sin_a, &mut cos_a, sin_step, cos_step);
        }
    }
}

impl AuroraApp {
    pub fn new() -> Self {
        let fields = vec![
            Field::new(1.0, 2.2, 22.0, 0.025, Color::Cyan),
            Field::new(1.7, 1.3, 18.0, 0.032, Color::Magenta),
        ];
        let ring = Ring::new(10.0, 2.8, 0.05, Color::Blue);
        let bounds = 24.0;

        Self {
            fields,
            ring,
            bounds,
            tick: 0,
        }
    }

    pub fn on_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        if self.tick % TICK_DIV != 0 {
            return;
        }
        for field in &mut self.fields {
            field.tick();
        }
        self.ring.tick();
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let mut datasets = Vec::with_capacity(self.fields.len() + 1);
        for field in &self.fields {
            datasets.push(
                Dataset::default()
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(field.color))
                    .data(&field.points),
            );
        }

        datasets.push(
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(self.ring.color))
                .data(&self.ring.points),
        );

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
