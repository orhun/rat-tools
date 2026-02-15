use alloc::{format, vec, vec::Vec};
use embedded_graphics::{
    image::Image,
    pixelcolor::Rgb565,
    prelude::{DrawTarget, OriginDimensions, Point},
    Drawable,
};
use libm::sinf;
use ratatui::widgets::canvas::{Line as CanvasLine, Rectangle as CanvasRect};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{
        canvas::Canvas, Bar, BarChart, Block, BorderType, Chart, Dataset, Gauge, GraphType,
        LineGauge, Padding, Paragraph, RatatuiLogo, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Sparkline, Table, Wrap,
    },
    Frame,
};
use tachyonfx::Duration as FxDuration;
use tui_big_text::{BigText, PixelSize};

use crate::effect::{DeckFx, EffectRegistry};
use crate::{
    bg::{aurora, hyper, nebula, waves},
    slides::{Background, ImagePosition, ImageSlide, Slide, TextSlide, TitleSlide, SLIDES},
    widget::CheeseMeter,
};

pub struct App {
    waves_app: waves::WavesApp,
    aurora_app: aurora::AuroraApp,
    nebula_app: nebula::NebulaApp,
    hyper_app: hyper::HyperApp,
    current_slide: usize,
    tick: u32,
    effect_registry: EffectRegistry,
}

impl App {
    pub fn new() -> Self {
        Self {
            waves_app: waves::WavesApp::new(),
            aurora_app: aurora::AuroraApp::new(),
            nebula_app: nebula::NebulaApp::new(),
            hyper_app: hyper::HyperApp::new(),
            current_slide: 0,
            tick: 0,
            effect_registry: EffectRegistry::new(),
        }
    }

    pub fn handle_button_press(&mut self) {
        self.next_slide();
    }

    pub fn next_slide(&mut self) {
        let len = SLIDES.len();
        if len == 0 {
            return;
        }
        let prev = self.current_slide;
        self.current_slide = (self.current_slide + 1) % len;

        self.update_effects_for_slide(prev);
    }

    pub fn prev_slide(&mut self) {
        let len = SLIDES.len();
        if len == 0 {
            return;
        }
        let prev = self.current_slide;
        self.current_slide = if self.current_slide == 0 {
            len - 1
        } else {
            self.current_slide - 1
        };

        self.update_effects_for_slide(prev);
    }

    fn update_effects_for_slide(&mut self, prev: usize) {
        let slide = &SLIDES[self.current_slide];
        let title = match slide {
            Slide::Title(TitleSlide { title, .. }) => *title,
            Slide::Text(TextSlide { title, .. }) => *title,
            Slide::Image(ImageSlide { title, .. }) => *title,
        };

        if self.is_image_like(prev)
            || self.is_image_like(self.current_slide)
            || slide.background().is_some()
        {
            if title == "<logo>" {
                self.effect_registry.register_logo_effect();
            } else if title.starts_with("<demo")
                || title == "<custom-widget>"
                || title == "<sponsor>"
                || title == "<let-him-cook>"
            {
                self.effect_registry.clear_effect(DeckFx::Transition)
            } else {
                self.effect_registry.register_transition();
            }
        } else {
            self.effect_registry.clear_effect(DeckFx::Transition);
        }

        // clear any existing bg effects, then register new ones if needed
        self.effect_registry.clear_effect(DeckFx::Bg);
        if let Slide::Title(TitleSlide { background, .. }) = slide {
            if [Background::Aurora, Background::Hyper].contains(background) {
                self.effect_registry.register_bg_effect()
            }
        }
    }

    fn is_image_like(&self, index: usize) -> bool {
        let Some(slide) = SLIDES.get(index) else {
            return false;
        };
        match slide {
            Slide::Image(_) => true,
            Slide::Title(TitleSlide { title, .. }) | Slide::Text(TextSlide { title, .. }) => {
                *title == "<intro2>" || *title == "<mascot>"
            }
        }
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
        let point = if *position == ImagePosition::Center || title == Some("<let-him-cook>") {
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

    pub fn render(&mut self, f: &mut Frame, elapsed_ms: u32) {
        self.tick = self.tick.wrapping_add(1);

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
        } else if title == Some("<intro2>") {
            self.intro_slide2(f);
        } else if title == Some("<mascot>") {
            self.mascot_slide(f);
        } else if title == Some("<ratdeck-title>") {
            let background = match slide {
                Slide::Title(TitleSlide { background, .. }) => *background,
                _ => Background::Waves,
            };
            self.ratdeck_title_slide(f, background);
        } else if title == Some("<qr-youtube>") {
            self.qr_youtube_slide(f);
        } else if title == Some("<qr-github>") {
            self.qr_github_slide(f);
        } else if title == Some("<sponsor-me>") {
            self.sponsor_me_slide(f);
        } else if title == Some("<questions>") {
            self.questions_slide(f);
        } else if title == Some("<logo>") {
            self.logo_slide(f);
        } else if title == Some("<demo-table-scrollbar>") {
            self.demo_table_scrollbar(f);
        } else if title == Some("<demo-sparkline>") {
            self.demo_sparkline(f);
        } else if title == Some("<demo-linegauge>") {
            self.demo_linegauge(f);
        } else if title == Some("<demo-gauge>") {
            self.demo_gauge(f);
        } else if title == Some("<demo-chart>") {
            self.demo_chart(f);
        } else if title == Some("<demo-canvas>") {
            self.demo_canvas(f);
        } else if title == Some("<demo-barchart>") {
            self.demo_barchart(f);
        } else if title == Some("<custom-widget>") {
            self.custom_widget_slide(f);
        } else if title == Some("<let-him-cook>") {
            // just let him cook
        } else {
            match slide {
                Slide::Title(data) => self.slide_with_title(f, data),
                Slide::Text(data) => self.slide_with_text(f, data),
                Slide::Image(data) => self.slide_with_image(f, data),
            }
        }

        // render effects, if we have them
        if self.effect_registry.has_active_effects() {
            let rect = f.area();
            self.effect_registry.process_effects(
                FxDuration::from_millis(elapsed_ms),
                f.buffer_mut(),
                rect,
            );
        }
    }

    fn slide_with_title(&mut self, f: &mut Frame, slide: &TitleSlide) {
        self.draw_background(f, slide.background);

        let text = Text::from(vec![Line::styled(
            format!("{}", slide.title),
            Style::new().white().on_black().bold(),
        )]);

        let area = f
            .area()
            .centered(Constraint::Percentage(80), Constraint::Percentage(20));

        f.render_widget(
            Paragraph::new(text)
                .wrap(Wrap { trim: false })
                .block(Block::bordered().style(Style::new().on_black())),
            area,
        );
    }

    fn draw_background(&mut self, f: &mut Frame, background: Background) {
        match background {
            Background::Waves => {
                self.waves_app.on_tick();
                self.waves_app.draw(f);
            }
            Background::Aurora => {
                // self.aurora_app.on_tick();
                self.aurora_app.draw(f);
            }
            Background::Nebula => {
                self.nebula_app.on_tick();
                self.nebula_app.draw(f);
            }
            Background::Hyper => {
                // self.hyper_app.on_tick();
                self.hyper_app.draw(f);
            }
        }
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
                Line::from_iter(["https://github.com/orhun/rat-tools".white().italic()]),
            ])),
            Rect {
                x: 0,
                y: f.area().height - 2,
                width: f.area().width,
                height: 2,
            },
        );
    }

    fn ratdeck_title_slide(&mut self, f: &mut Frame, background: Background) {
        self.draw_background(f, background);

        let title = BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .style(Style::new().green())
            .lines(vec!["".into(), "".into(), " RATDECK".white().into()])
            .build();

        f.render_widget(
            title,
            Rect {
                x: 0,
                y: 0,
                width: f.area().width,
                height: f.area().height,
            },
        );
    }

    fn qr_youtube_slide(&mut self, f: &mut Frame) {
        let text = Text::from(vec![
            Line::from("    █▀▀▀▀▀█ █ ▄ ▀███ █▄▄  █▀▀▀▀▀█"),
            Line::from("    █ ███ █ ▀  ▄█▀█▄ ▄▀▄█ █ ███ █"),
            Line::from("    █ ▀▀▀ █ ▀▀▄▀ █▄▀ ▄█▀  █ ▀▀▀ █"),
            Line::from("    ▀▀▀▀▀▀▀ ▀▄▀▄▀ █ ▀ █▄█ ▀▀▀▀▀▀▀"),
            Line::from("    █  ▀█▀▀▀▀▀▄  ▀█▀▀▀ ▀ █ ▄█ ██▀"),
            Line::from("    ▀▄ ▀▄█▀▀█ █▄▄ █▄ █ ▄▄▄▄▀█▄▀ ▄"),
            Line::from("    ▀▀█ █▀▀▄█▄▄▄ ▀▀  ▄▄ █ █▄▄▄▄▄█"),
            Line::from("    ▀   ▀ ▀ █▄▀▀█▄█▀▄▀▄▀▀▀▀██ █ █"),
            Line::from("    ▄▀█▀█▀▀█▀▄▀ ▀  ▀ ▄▄██   ▄▀▄▄"),
            Line::from("    ██▄ ██▀▄▀██▄██▀▄█▀▄▀ ▄▀▄ █▄ ▀"),
            Line::from("    ▀▀▀   ▀▀█▄██▄█  ▄▀▄ █▀▀▀██▀▀"),
            Line::from("    █▀▀▀▀▀█ █▀▄▀▄█ ▀██ ██ ▀ █▀ ▄▄"),
            Line::from("    █ ███ █ ██   ▀▄▀█  ▀▀▀▀▀▀▀ ▄"),
            Line::from("    █ ▀▀▀ █   ▄▀ █ ▄▀▀▄▀ █ ██▄█▀█"),
            Line::from("    ▀▀▀▀▀▀▀ ▀        ▀▀▀▀▀▀▀ ▀"),
        ]);

        let area = f
            .area()
            .centered(Constraint::Percentage(95), Constraint::Percentage(90));
        f.render_widget(
            Paragraph::new(text)
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: false })
                .block(Block::default()),
            area,
        );
    }

    fn qr_github_slide(&mut self, f: &mut Frame) {
        let text = Text::from(vec![
            Line::from("    █▀▀▀▀▀█ ▀▄▀█▀ █ ▄ █▀▀▀▀▀█"),
            Line::from("    █ ███ █ ▀▄█▄▄█ ▄  █ ███ █"),
            Line::from("    █ ▀▀▀ █ ▄▀ ▄█▄█ ▄ █ ▀▀▀ █"),
            Line::from("    ▀▀▀▀▀▀▀ ▀ ▀▄▀ █▄▀ ▀▀▀▀▀▀▀"),
            Line::from("    ▀▄▀ ▄ ▀█▄▄▄█▀▄ ██ ▄█ ▄▀▄█"),
            Line::from("     ▀▀  ▄▀ ██▀▄  ▄███ ▄ █▀ ▀"),
            Line::from("    ▀▀ ▀▀█▀▄▀ ▀   ▄▄▀▄██   ▄█"),
            Line::from("    ▀▀██▀█▀ ▄▀▀▀  ▀█▀ ▀▄▄█▀ ▀"),
            Line::from("    ▀▀  ▀▀▀▀▄ ▄▀█▄ ▀█▀▀▀█  ▀▄"),
            Line::from("    █▀▀▀▀▀█ ▀▄▄▄█▀▀▄█ ▀ █   ▀"),
            Line::from("    █ ███ █  ▄█ ▀ ▄█▀█▀▀█ ▄█"),
            Line::from("    █ ▀▀▀ █ ▀█▀██▀▄▄█▀▄██▀ ▀▀"),
            Line::from("    ▀▀▀▀▀▀▀ ▀ ▀ ▀   ▀▀▀  ▀  ▀"),
        ]);

        let area = f
            .area()
            .centered(Constraint::Percentage(85), Constraint::Percentage(80));
        f.render_widget(
            Paragraph::new(text)
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: false })
                .block(Block::default()),
            area,
        );
    }

    fn sponsor_me_slide(&mut self, f: &mut Frame) {
        let text = BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .lines(vec![
                "HIRE ME /".green().into(),
                "SPONSOR".yellow().into(),
                "ME".yellow().into(),
            ])
            .build();

        let area = f
            .area()
            .centered(Constraint::Percentage(95), Constraint::Percentage(80));
        f.render_widget(text, area);

        f.render_widget(
            Paragraph::new(Text::from(vec![Line::from("github.com/orhun").cyan()]))
                .alignment(Alignment::Center),
            Rect {
                x: 0,
                y: f.area().height.saturating_sub(2),
                width: f.area().width,
                height: 2,
            },
        );
    }

    fn questions_slide(&mut self, f: &mut Frame) {
        let text = BigText::builder()
            .pixel_size(PixelSize::Full)
            .lines(vec![" Q&A".white().into()])
            .build();

        let area = f
            .area()
            .centered(Constraint::Percentage(95), Constraint::Percentage(80));
        f.render_widget(text, area);

        f.render_widget(
            Paragraph::new(Text::from(vec![Line::from(
                "P.S. There isn't a rat in this device.",
            )
            .yellow()]))
            .alignment(Alignment::Center),
            Rect {
                x: 0,
                y: f.area().height.saturating_sub(2),
                width: f.area().width,
                height: 2,
            },
        );
    }

    fn logo_slide(&mut self, f: &mut Frame) {
        let content_area = f
            .area()
            .centered(Constraint::Length(31), Constraint::Length(8));
        let [content_block_area, ratatui_url_area, mousefood_url_area] = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(content_area);

        let block = Block::bordered()
            .padding(Padding::uniform(1))
            .border_style(Color::Yellow);
        let logo_area = block.inner(content_block_area);
        f.render_widget(block, content_block_area);
        f.render_widget(RatatuiLogo::small(), logo_area);
        f.render_widget(
            "github.com/ratatui/ratatui".gray().underlined(),
            ratatui_url_area,
        );
        f.render_widget(
            "github.com/ratatui/mousefood".gray().underlined(),
            mousefood_url_area,
        );
    }

    fn mascot_slide(&self, f: &mut Frame) {
        let mascot_str = [
            "",
            "                ▄▄███              ",
            "              ▄███████             ",
            "            ▄█████████             ",
            "           ████████████            ",
            "           ▀███████████▀   ▄▄██████",
            "                 ▀███▀▄█▀▀████████ ",
            "               ▄▄▄▄▀▄████████████  ",
            "              ████████████████     ",
            "              ▀███▀██████████      ",
            "            ▄▀▀▄   █████████       ",
            "          ▄▀ ▄  ▀▄▀█████████       ",
            "        ▄▀  ▀▀    ▀▄▀███████       ",
            "      ▄▀      ▄▄    ▀▄▀█████████   ",
            "    ▄▀         ▀▀     ▀▄▀██▀  ███  ",
            "   █                    ▀▄▀  ▄██   ",
            "    ▀▄                    ▀▄▀█     ",
        ];
        let mascot = mascot_str
            .iter()
            .map(|line| Line::styled(*line, Style::new().white()))
            .collect::<Vec<Line>>();
        f.render_widget(Paragraph::new(mascot), f.area());
    }

    fn demo_table_scrollbar(&self, f: &mut Frame) {
        let area = f.area().inner(Margin {
            horizontal: 2,
            vertical: 1,
        });
        let chunks = Layout::horizontal([Constraint::Min(0), Constraint::Length(2)]).split(area);
        let table_area = chunks[0];
        let scrollbar_area = chunks[1];

        let total_rows = 30usize;
        let visible = table_area.height.saturating_sub(2) as usize;
        let visible = visible.max(1);
        let max_offset = total_rows.saturating_sub(visible).max(1);
        let offset = (self.tick as usize / 3) % max_offset;

        let mut rows = Vec::new();
        for i in offset..(offset + visible).min(total_rows) {
            rows.push(Row::new(vec![
                format!("Row {i:02}"),
                format!("v{}", (self.tick + i as u32) % 100),
            ]));
        }

        let table = Table::new(rows, [Constraint::Length(10), Constraint::Min(0)])
            .block(Block::bordered().title("Table + Scrollbar").white())
            .column_spacing(1);
        f.render_widget(table, table_area);

        let mut state = ScrollbarState::new(total_rows)
            .position(offset)
            .viewport_content_length(visible);
        let scrollbar =
            Scrollbar::new(ScrollbarOrientation::VerticalRight).thumb_style(Style::new().yellow());
        f.render_stateful_widget(scrollbar, scrollbar_area, &mut state);
    }

    fn demo_sparkline(&self, f: &mut Frame) {
        let area = f.area().inner(Margin {
            horizontal: 2,
            vertical: 2,
        });
        let data: Vec<u64> = (0..60)
            .map(|i| {
                let x = (self.tick as f32 / 8.0) + i as f32 / 4.0;
                let v = (sinf(x) * 0.5 + 0.5) * 100.0;
                v as u64
            })
            .collect();
        let sparkline = Sparkline::default()
            .block(Block::bordered().title("Sparkline").white())
            .data(data)
            .style(Style::new().cyan());
        f.render_widget(sparkline, area);
    }

    fn demo_linegauge(&self, f: &mut Frame) {
        let area = f.area().inner(Margin {
            horizontal: 2,
            vertical: 3,
        });
        let percent = (self.tick % 100) as u16;
        let ratio = percent as f64 / 100.0;
        let gauge = LineGauge::default()
            .block(Block::bordered().title("LineGauge").white())
            .ratio(ratio)
            .label(Line::from(format!("Load {percent}%")).white())
            .filled_style(Style::new().green())
            .unfilled_style(Style::new().dark_gray());
        f.render_widget(gauge, area);
    }

    fn demo_gauge(&self, f: &mut Frame) {
        let area = f.area().inner(Margin {
            horizontal: 2,
            vertical: 3,
        });
        let percent = (self.tick % 100) as u16;
        let ratio = percent as f64 / 100.0;
        let gauge = Gauge::default()
            .block(Block::bordered().title("Gauge").white())
            .ratio(ratio)
            .gauge_style(Style::new().green().on_black());
        f.render_widget(gauge, area);
    }

    fn demo_chart(&self, f: &mut Frame) {
        let area = f.area().inner(Margin {
            horizontal: 2,
            vertical: 2,
        });
        let mut data = Vec::with_capacity(50);
        for i in 0..50 {
            let x = i as f64;
            let y = (sinf((i as f32 / 5.0) + (self.tick as f32 / 10.0)) * 4.0 + 5.0) as f64;
            data.push((x, y));
        }
        let dataset = Dataset::default()
            .name("cheese")
            .graph_type(GraphType::Line)
            .data(&data)
            .style(Style::new().yellow());
        let chart = Chart::new(vec![dataset])
            .block(Block::bordered().title("Chart").white())
            .x_axis(ratatui::widgets::Axis::default().bounds([0.0, 50.0]))
            .y_axis(ratatui::widgets::Axis::default().bounds([0.0, 10.0]));
        f.render_widget(chart, area);
    }

    fn demo_canvas(&self, f: &mut Frame) {
        let area = f.area().inner(Margin {
            horizontal: 2,
            vertical: 2,
        });
        let t = (self.tick % 100) as f64;
        let canvas = Canvas::default()
            .block(Block::bordered().title("Canvas").white())
            .x_bounds([0.0, 100.0])
            .y_bounds([0.0, 100.0])
            .paint(|ctx| {
                ctx.draw(&CanvasRect {
                    x: 10.0,
                    y: 10.0,
                    width: 30.0,
                    height: 20.0,
                    color: Color::Yellow,
                });
                ctx.draw(&CanvasLine::new(0.0, 50.0, t, 80.0, Color::Cyan));
                ctx.draw(&CanvasLine::new(
                    100.0 - t,
                    20.0,
                    100.0,
                    20.0 + t / 2.0,
                    Color::Green,
                ));
            });
        f.render_widget(canvas, area);
    }

    fn demo_barchart(&self, f: &mut Frame) {
        let area = f.area().inner(Margin {
            horizontal: 2,
            vertical: 2,
        });
        let base = (self.tick % 50) as u64;
        let bars = vec![
            Bar::with_label("A", 10 + base),
            Bar::with_label("B", 30 + (base / 2)),
            Bar::with_label("C", 20 + (base / 3)),
            Bar::with_label("D", 40 + (base / 4)),
            Bar::with_label("E", 15 + (base / 5)),
        ];
        let chart = BarChart::new(bars)
            .block(Block::bordered().title("BarChart").white())
            .bar_width(3)
            .bar_gap(1)
            .bar_style(Style::new().magenta());
        f.render_widget(chart, area);
    }

    fn custom_widget_slide(&self, f: &mut Frame) {
        let area = f.area().inner(Margin {
            horizontal: 2,
            vertical: 2,
        });

        let block = Block::bordered()
            .title("┤ Custom widget ├".white())
            .border_type(BorderType::Rounded);
        f.render_widget(block, area);

        let inner = area.inner(Margin {
            horizontal: 2,
            vertical: 2,
        });

        let widget = CheeseMeter {
            label: "Cheese",
            value: 42,
        };
        f.render_widget(widget, inner);
    }
}
