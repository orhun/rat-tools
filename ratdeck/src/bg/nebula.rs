use alloc::vec::Vec;

use alloc::vec;
use libm::{cos, sin};
use ratatui::prelude::*;
use ratatui::widgets::{Axis, Chart, Dataset, GraphType};

const ARM_POINTS: usize = 220;
const ARM_SPAN: f64 = 18.0;
const Y_SCALE: f64 = 0.7;

pub struct NebulaApp {
    arms: Vec<Arm>,
    bounds: f64,
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
        for (idx, point) in self.points.iter_mut().enumerate() {
            let t = idx as f64 / (ARM_POINTS - 1) as f64;
            let angle = self.phase + t * self.twist;
            let radius = self.radius + t * ARM_SPAN;
            let wobble = sin(self.phase * 0.7 + t * 8.0) * 0.8;
            let x = cos(angle) * (radius + wobble);
            let y = sin(angle) * (radius + wobble) * Y_SCALE;
            *point = (x, y);
        }
    }
}

impl NebulaApp {
    pub fn new() -> Self {
        let arms = vec![
            Arm::new(0.0, 0.03, 8.0, 2.0, Color::LightBlue),
            Arm::new(2.1, 0.028, 7.6, 2.5, Color::Magenta),
            Arm::new(4.2, 0.026, 7.9, 1.5, Color::Yellow),
        ];
        let bounds = 22.0;

        Self { arms, bounds }
    }

    pub fn on_tick(&mut self) {
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
