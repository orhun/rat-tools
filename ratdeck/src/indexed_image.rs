#![allow(dead_code)]

use embedded_graphics::geometry::Size;
use embedded_graphics::image::ImageDrawable;
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;

pub struct IndexedImage {
    indices: &'static [u8],
    palette: &'static [u16; 256],
    size: Size,
}

impl IndexedImage {
    pub const fn new(indices: &'static [u8], palette: &'static [u16; 256], width: u32) -> Self {
        let height = indices.len() as u32 / width;
        Self {
            indices,
            palette,
            size: Size::new(width, height),
        }
    }

    #[inline]
    fn lookup(&self, index: u8) -> Rgb565 {
        Rgb565::from(RawU16::new(self.palette[index as usize]))
    }
}

impl OriginDimensions for IndexedImage {
    fn size(&self) -> Size {
        self.size
    }
}

impl ImageDrawable for IndexedImage {
    type Color = Rgb565;

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        target.fill_contiguous(
            &self.bounding_box(),
            self.indices.iter().map(|&i| self.lookup(i)),
        )
    }

    fn draw_sub_image<D>(&self, target: &mut D, area: &Rectangle) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let area = area.intersection(&Rectangle::new(Point::zero(), self.size));
        if area.is_zero_sized() {
            return Ok(());
        }

        let width = self.size.width as usize;
        let sub_w = area.size.width as usize;
        let x0 = area.top_left.x as usize;
        let y0 = area.top_left.y as usize;

        let colors = (0..area.size.height as usize).flat_map(move |row| {
            let start = (y0 + row) * width + x0;
            self.indices[start..start + sub_w]
                .iter()
                .map(|&i| self.lookup(i))
        });

        target.fill_contiguous(&area, colors)
    }
}
