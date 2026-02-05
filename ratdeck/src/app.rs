use alloc::vec;
use embedded_graphics::{
    image::Image,
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
    Drawable,
};
use ratatui::{
    layout::{Alignment, Margin, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, Paragraph},
    Frame,
};
use tachyonfx::{fx, Duration as FxDuration, Effect, EffectRenderer};
use tui_big_text::{BigText, PixelSize};

use crate::chart;

pub struct App {
    chart_app: chart::ChartApp,
    current_slide: usize,
    effect: Effect,
}

impl App {
    pub fn new() -> Self {
        Self {
            chart_app: chart::ChartApp::new(),
            current_slide: 0,
            effect: Self::get_effect(),
        }
    }

    // TODO: Pick random effects
    // <https://junkdog.github.io/tachyonfx-ftl>
    fn get_effect() -> Effect {
        fx::explode(10.0, 3.0, 1000)
    }

    pub fn handle_button_press(&mut self) {
        self.current_slide += 1;
        self.effect = Self::get_effect();
    }

    pub fn render_image<D>(&mut self, display: &mut D)
    where
        D: DrawTarget<Color = Rgb565>,
        D::Error: core::fmt::Debug,
    {
        if self.current_slide == 1 {
            let im = Image::new(&crate::assets::RAT_CHEF, Point::new(0, 10));
            im.draw(display).unwrap();
        }
    }

    pub fn render(&mut self, f: &mut Frame) {
        if self.current_slide == 0 {
            self.intro_slide1(f);
        }

        if self.current_slide == 1 {
            self.intro_slide2(f);
        }

        if !self.effect.done() && self.current_slide != 1 {
            f.render_effect(&mut self.effect, f.area(), FxDuration::from_millis(100));
        }
    }

    fn generic_slide(&mut self, f: &mut Frame) {}

    fn intro_slide2(&mut self, f: &mut Frame) {
        f.render_widget(
            Paragraph::new(Text::from(vec![
                Line::styled("Orhun", Style::new().white().bold()),
                Line::styled("Parmaksız", Style::new().white().bold()),
                Line::styled("from Turkey", Style::new().red()),
                Line::default(),
                Line::styled("Terminal Chef", Style::new().green()),
                Line::styled("@ Ratatui", Style::new().green().bold()),
                Line::default(),
                Line::styled("Package", Style::new().cyan()),
                Line::styled("Maintainer", Style::new().cyan()),
                Line::styled("@ Arch Linux", Style::new().cyan().bold()),
            ]))
            .block(
                Block::bordered()
                    .title("| who u? |".white())
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().white())
                    .title_bottom("orhun.dev")
                    .title_alignment(Alignment::Center),
            ),
            Rect {
                x: f.area().width / 2,
                y: 0,
                width: f.area().width / 2,
                height: f.area().height,
            }
            .inner(Margin {
                horizontal: 2,
                vertical: 3,
            }),
        );
    }

    fn intro_slide1(&mut self, f: &mut Frame) {
        // self.chart_app.on_tick();
        // self.chart_app.draw(f);
        let first_line = BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .style(Style::new().green())
            .lines(vec![
                "If it can".white().into(),
                "compute,".white().into(),
                "it can run:".white().into(),
                "Ratatui".green().into(),
            ])
            .build();

        f.render_widget(
            first_line,
            Rect {
                x: 0,
                y: 0,
                width: f.area().width,
                height: f.area().height,
            },
        );

        f.render_widget(
            Paragraph::new(Text::from(vec![
                Line::from_iter([
                    "Orhun Parmaksız".cyan(),
                    " | ".white(),
                    "RustNation UK 2026".magenta(),
                ]),
                Line::from_iter(["https://github.com/orhun/rustnation2026".white().italic()]),
            ])),
            Rect {
                x: 0,
                y: f.area().height - 2,
                width: f.area().width,
                height: 2,
            },
        );
    }
}
