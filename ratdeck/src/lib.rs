#![no_std]

extern crate alloc;

pub use mousefood::fonts::atlas;

pub mod app;
pub mod assets;
pub mod indexed_image;
pub mod bg;
mod effect;
pub mod font_8x13;
#[allow(non_snake_case)]
pub mod font_8x13B;
pub mod slides;
pub mod widget;
