use alloc::vec::Vec;

use alloc::vec;
use libm::{cos, sin};
use ratatui::prelude::*;
use ratatui::widgets::{Axis, Chart, Dataset, GraphType};

const ARM_POINTS: usize = 120;
const ARM_SPAN: f64 = 18.0;
const Y_SCALE: f64 = 0.2;
const TICK_DIV: u32 = 1;

#[inline]
fn advance_sin_cos(sin_v: &mut f64, cos_v: &mut f64, sin_delta: f64, cos_delta: f64) {
    let next_sin = *sin_v * cos_delta + *cos_v * sin_delta;
    let next_cos = *cos_v * cos_delta - *sin_v * sin_delta;
    *sin_v = next_sin;
    *cos_v = next_cos;
}

pub struct NebulaApp {
    arms: Vec<Arm>,
    bounds: f64,
    tick: u32,
}

struct Arm {
    points: Vec<(f64, f64)>,
    phase: f64,
    speed: f64,
    twist: f64,
    radius: f64,
    color: Color,
}

impl Arm {
    fn new(phase: f64, speed: f64, twist: f64, radius: f64, color: Color) -> Self {
        let mut arm = Self {
            points: vec![(0.0, 0.0); ARM_POINTS],
            phase,
            speed,
            twist,
            radius,
            color,
        };
        arm.recompute();
        arm
    }

    fn tick(&mut self) {
        self.phase += self.speed;
        self.recompute();
    }

    fn recompute(&mut self) {
        let steps = (self.points.len() - 1) as f64;
        let t_step = 1.0 / steps;
        let angle_step = self.twist * t_step;
        let wobble_step = 8.0 * t_step;
        let (sin_da, cos_da) = (sin(angle_step), cos(angle_step));
        let (sin_dw, cos_dw) = (sin(wobble_step), cos(wobble_step));
        let (mut sin_a, mut cos_a) = (sin(self.phase), cos(self.phase));
        let (mut sin_w, mut cos_w) = (sin(self.phase * 0.7), cos(self.phase * 0.7));
        let mut radius = self.radius;
        let radius_step = ARM_SPAN * t_step;
        for point in &mut self.points {
            let wobble = sin_w * 0.8;
            let r = radius + wobble;
            let x = cos_a * r;
            let y = sin_a * r * Y_SCALE;
            *point = (x, y);
            radius += radius_step;
            advance_sin_cos(&mut sin_a, &mut cos_a, sin_da, cos_da);
            advance_sin_cos(&mut sin_w, &mut cos_w, sin_dw, cos_dw);
        }
    }
}

impl NebulaApp {
    pub fn new() -> Self {
        let arms = vec![
            Arm::new(0.0, 0.3, 8.0, 2.0, Color::Cyan),
            Arm::new(2.1, 0.3, 7.6, 2.5, Color::Green),
        ];
        let bounds = 22.0;

        Self {
            arms,
            bounds,
            tick: 0,
        }
    }

    pub fn on_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        if self.tick % TICK_DIV != 0 {
            return;
        }
        for arm in &mut self.arms {
            arm.tick();
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let mut datasets = Vec::with_capacity(self.arms.len());
        for arm in &self.arms {
            datasets.push(
                Dataset::default()
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(arm.color))
                    .data(&arm.points),
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
