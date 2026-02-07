use alloc::vec::Vec;

use alloc::vec;
use libm::{cos, sin};
use ratatui::prelude::*;
use ratatui::widgets::{Axis, Chart, Dataset, GraphType};

const FIELD_POINTS: usize = 260;
const RING_POINTS: usize = 180;
const STEP: f64 = 0.08;
const Y_SCALE: f64 = 0.6;

pub struct AuroraApp {
    fields: Vec<Field>,
    ring: Ring,
    bounds: f64,
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
        for (idx, point) in self.points.iter_mut().enumerate() {
            let t = idx as f64 * STEP;
            let x = cos(t * self.freq_x + self.phase) * self.scale;
            let y = sin(t * self.freq_y + self.phase * 1.3) * self.scale * Y_SCALE;
            *point = (x, y);
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
        for (idx, point) in self.points.iter_mut().enumerate() {
            let a = idx as f64 * step;
            let x = cos(a) * radius;
            let y = sin(a) * radius * Y_SCALE;
            *point = (x, y);
        }
    }
}

impl AuroraApp {
    pub fn new() -> Self {
        let fields = vec![
            Field::new(1.0, 2.2, 22.0, 0.025, Color::Cyan),
            Field::new(1.7, 1.3, 18.0, 0.032, Color::Magenta),
            Field::new(2.4, 0.9, 14.0, 0.041, Color::Yellow),
        ];
        let ring = Ring::new(10.0, 2.8, 0.05, Color::Blue);
        let bounds = 24.0;

        Self {
            fields,
            ring,
            bounds,
        }
    }

    pub fn on_tick(&mut self) {
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
