use ratatui::text::Text;

#[derive(Debug)]
pub enum Slide {
    Title(TitleSlide),
    Text(TextSlide),
    Image(ImageSlide),
}

#[derive(Debug)]
pub struct TitleSlide {
    pub title: &'static str,
    pub background: Background,
}

#[derive(Debug)]
pub struct TextSlide {
    pub title: &'static str,
    pub text: &'static Text<'static>,
}

#[derive(Debug)]
pub struct ImageSlide {
    pub title: &'static str,
    pub image: &'static str,
    pub position: ImagePosition,
    pub width: u32,
    pub height: u32,
    pub text: &'static Text<'static>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImagePosition {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Background {
    Waves,
    Aurora,
    Nebula,
    Hyper,
}

include!(concat!(env!("OUT_DIR"), "/slides.rs"));
