use alloc::vec::Vec;

use alloc::vec;
use libm::sin;
use ratatui::prelude::*;
use ratatui::widgets::{Axis, Chart, Dataset};

pub struct WavesApp {
    signal1: SinSignal,
    data1: Vec<(f64, f64)>,
    signal2: SinSignal,
    data2: Vec<(f64, f64)>,
    window: [f64; 2],
}

#[derive(Clone)]
struct SinSignal {
    x: f64,
    interval: f64,
    period: f64,
    scale: f64,
}

impl SinSignal {
    const fn new(interval: f64, period: f64, scale: f64) -> Self {
        Self {
            x: 0.0,
            interval,
            period,
            scale,
        }
    }
}

impl Iterator for SinSignal {
    type Item = (f64, f64);
    fn next(&mut self) -> Option<Self::Item> {
        let point = (self.x, sin(self.x / self.period) * self.scale);
        self.x += self.interval;
        Some(point)
    }
}

impl WavesApp {
    pub fn new() -> Self {
        let mut signal1 = SinSignal::new(0.2, 3.0, 18.0);
        let mut signal2 = SinSignal::new(0.1, 2.0, 10.0);
        let data1 = signal1.by_ref().take(200).collect::<Vec<(f64, f64)>>();
        let data2 = signal2.by_ref().take(200).collect::<Vec<(f64, f64)>>();

        Self {
            signal1,
            data1,
            signal2,
            data2,
            window: [0.0, 20.0],
        }
    }

    pub fn on_tick(&mut self) {
        self.data1.drain(0..5);
        self.data1.extend(self.signal1.by_ref().take(5));

        self.data2.drain(0..10);
        self.data2.extend(self.signal2.by_ref().take(10));

        self.window[0] += 1.0;
        self.window[1] += 1.0;
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let datasets = vec![
            Dataset::default()
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::Cyan))
                .data(&self.data1),
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Green))
                .data(&self.data2),
        ];

        let chart = Chart::new(datasets)
            .x_axis(
                Axis::default()
                    .style(Style::default().fg(Color::Black))
                    .bounds(self.window),
            )
            .y_axis(
                Axis::default()
                    .style(Style::default().fg(Color::Black))
                    .bounds([-20.0, 20.0]),
            );

        frame.render_widget(chart, frame.area());
    }
}
