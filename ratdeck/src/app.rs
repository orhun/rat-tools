use alloc::{format, vec};
use embedded_graphics::{
    image::Image,
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
    Drawable,
};
use ratatui::{
    layout::{Alignment, Constraint, Margin, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, Paragraph, Wrap},
    Frame,
};
use tachyonfx::{fx, Duration as FxDuration, Effect, EffectRenderer};
use tui_big_text::{BigText, PixelSize};

use crate::{
    chart,
    slides::{ImagePosition, ImageSlide, Slide, TextSlide, TitleSlide, SLIDES},
};

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
        self.next_slide();
    }

    pub fn next_slide(&mut self) {
        let len = SLIDES.len();
        if len == 0 {
            return;
        }
        self.current_slide = (self.current_slide + 1) % len;
        self.effect = Self::get_effect();
    }

    pub fn prev_slide(&mut self) {
        let len = SLIDES.len();
        if len == 0 {
            return;
        }
        if self.current_slide == 0 {
            self.current_slide = len - 1;
        } else {
            self.current_slide -= 1;
        }
        self.effect = Self::get_effect();
    }

    pub fn render_image<D>(&mut self, display: &mut D)
    where
        D: DrawTarget<Color = Rgb565>,
        D::Error: core::fmt::Debug,
    {
        let Some(slide) = SLIDES.get(self.current_slide) else {
            return;
        };

        let title = match slide {
            Slide::Title(TitleSlide { title }) => Some(*title),
            Slide::Text(TextSlide { title, .. }) => Some(*title),
            Slide::Image(ImageSlide { title, .. }) => Some(*title),
        };

        if title == Some("<intro2>") {
            let im = Image::new(&crate::assets::RAT_CHEF, Point::new(0, 10));
            im.draw(display).unwrap();
            return;
        }

        let image_name = match slide {
            Slide::Image(ImageSlide { image, .. }) => Some(*image),
            _ => None,
        };

        let Some(image_name) = image_name else {
            return;
        };
        let Some(image) = crate::assets::resolve_image(image_name) else {
            return;
        };

        let im = Image::new(image, Point::new(0, 0));
        im.draw(display).unwrap();
    }

    pub fn render(&mut self, f: &mut Frame) {
        let Some(slide) = SLIDES.get(self.current_slide) else {
            return;
        };

        let title = match slide {
            Slide::Title(TitleSlide { title }) => Some(*title),
            Slide::Text(TextSlide { title, .. }) => Some(*title),
            Slide::Image(ImageSlide { title, .. }) => Some(*title),
        };

        if title == Some("<intro1>") {
            self.intro_slide1(f);
            return;
        } else if title == Some("<intro2>") {
            self.intro_slide2(f);
            return;
        }

        match slide {
            Slide::Title(data) => self.slide_with_title(f, data),
            Slide::Text(data) => self.slide_with_text(f, data),
            Slide::Image(data) => self.slide_with_image(f, data),
        }

        if !self.effect.done() {
            f.render_effect(&mut self.effect, f.area(), FxDuration::from_millis(100));
        }
    }

    fn slide_with_title(&mut self, f: &mut Frame, slide: &TitleSlide) {
        self.chart_app.on_tick();
        self.chart_app.draw(f);

        let text = Text::from(vec![Line::styled(
            slide.title,
            Style::new().black().on_white(),
        )]);

        let area = f
            .area()
            .centered(Constraint::Percentage(80), Constraint::Percentage(20));

        f.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), area);
    }

    fn slide_with_text(&mut self, f: &mut Frame, slide: &TextSlide) {
        let mut text = slide.text.clone();
        text.lines.insert(0, Line::default());
        f.render_widget(
            Paragraph::new(text)
                .block(
                    Block::bordered()
                        .title(format!("┤ {} ├", slide.title).white())
                        .border_type(BorderType::Rounded),
                )
                .wrap(Wrap { trim: false }),
            f.area().inner(Margin {
                horizontal: 1,
                vertical: 1,
            }),
        );
    }

    fn slide_with_image(&mut self, f: &mut Frame, slide: &ImageSlide) {
        if slide.position == ImagePosition::Center {
            return;
        }
        let mut text = slide.text.clone();
        text.lines.insert(0, Line::default());
        f.render_widget(
            Paragraph::new(text)
                .block(
                    Block::bordered()
                        .title(format!("┤ {} ├", slide.title).white())
                        .border_type(BorderType::Rounded),
                )
                .wrap(Wrap { trim: false }),
            Rect {
                x: f.area().width / 2,
                y: 0,
                width: f.area().width / 2,
                height: f.area().height,
            }
            .inner(Margin {
                horizontal: 1,
                vertical: 2,
            }),
        );
    }

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
                    .title("┤ rat bio ├".white())
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
