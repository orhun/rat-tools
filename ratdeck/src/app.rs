use alloc::{format, vec};
use embedded_graphics::{
    image::Image,
    pixelcolor::Rgb565,
    prelude::{DrawTarget, OriginDimensions, Point},
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
    bg::{aurora, hyper, nebula, waves},
    slides::{Background, ImagePosition, ImageSlide, Slide, TextSlide, TitleSlide, SLIDES},
};

pub struct App {
    waves_app: waves::WavesApp,
    aurora_app: aurora::AuroraApp,
    nebula_app: nebula::NebulaApp,
    hyper_app: hyper::HyperApp,
    current_slide: usize,
    effect: Effect,
}

impl App {
    pub fn new() -> Self {
        Self {
            waves_app: waves::WavesApp::new(),
            aurora_app: aurora::AuroraApp::new(),
            nebula_app: nebula::NebulaApp::new(),
            hyper_app: hyper::HyperApp::new(),
            current_slide: SLIDES.len() - 1, // TODO: Start from the first slide
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
        D: DrawTarget<Color = Rgb565> + OriginDimensions,
        D::Error: core::fmt::Debug,
    {
        let Some(slide) = SLIDES.get(self.current_slide) else {
            return;
        };

        let title = match slide {
            Slide::Title(TitleSlide { title, .. }) => Some(*title),
            Slide::Text(TextSlide { title, .. }) => Some(*title),
            Slide::Image(ImageSlide { title, .. }) => Some(*title),
        };

        if title == Some("<intro2>") {
            let im = Image::new(&crate::assets::RAT_CHEF, Point::new(0, 10));
            im.draw(display).unwrap();
            return;
        }

        let (image_name, image_width, image_height, position) = match slide {
            Slide::Image(ImageSlide {
                image,
                width,
                height,
                position,
                ..
            }) => (image, *width, *height, position),
            _ => return,
        };

        let Some(image) = crate::assets::resolve_image(image_name) else {
            return;
        };

        let display_size = display.size();
        let point = if *position == ImagePosition::Center {
            Point::new(
                (display_size.width.saturating_sub(image_width) / 2) as i32,
                (display_size.height.saturating_sub(image_height) / 2) as i32,
            )
        } else {
            Point::new(0, 0)
        };

        let im = Image::new(image, point);
        im.draw(display).unwrap();
    }

    pub fn render(&mut self, f: &mut Frame) {
        let Some(slide) = SLIDES.get(self.current_slide) else {
            return;
        };

        let title = match slide {
            Slide::Title(TitleSlide { title, .. }) => Some(*title),
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
        match slide.background {
            Background::Waves => {
                self.waves_app.on_tick();
                self.waves_app.draw(f);
            }
            Background::Aurora => {
                self.aurora_app.on_tick();
                self.aurora_app.draw(f);
            }
            Background::Nebula => {
                self.nebula_app.on_tick();
                self.nebula_app.draw(f);
            }
            Background::Hyper => {
                self.hyper_app.on_tick();
                self.hyper_app.draw(f);
            }
        }

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
