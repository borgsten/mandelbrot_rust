use sdl2::pixels::Color;
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy)]
pub enum ColorAlg {
    DEFAULT,
    LCH,
    HSV,
    HSL,
    BW,
}

impl ColorAlg {
    pub fn next(&self) -> ColorAlg {
        match self {
            ColorAlg::DEFAULT => ColorAlg::LCH,
            ColorAlg::LCH => ColorAlg::HSV,
            ColorAlg::HSV => ColorAlg::HSL,
            ColorAlg::HSL => ColorAlg::BW,
            ColorAlg::BW => ColorAlg::DEFAULT,
        }
    }

    pub fn get_rgb_color(&self, iteration_ratio: f32) -> Color {
        match self {
            ColorAlg::DEFAULT => ColorAlg::default(iteration_ratio),
            ColorAlg::BW => ColorAlg::bw(iteration_ratio),
            ColorAlg::LCH => ColorAlg::lch(iteration_ratio),
            ColorAlg::HSV => ColorAlg::hsv(iteration_ratio),
            ColorAlg::HSL => ColorAlg::hsl(iteration_ratio),
        }
    }

    fn palette_to_sdl_color<T: Into<palette::Rgb>>(palette: T) -> Color {
        let rgb: palette::Rgb = palette.into();
        Color::RGBA(
            (rgb.red * 255.0) as u8,
            (rgb.green * 255.0) as u8,
            (rgb.blue * 255.0) as u8,
            (rgb.alpha * 255.0) as u8,
        )
    }

    fn bw(iteration_ratio: f32) -> Color {
        if iteration_ratio > 0.99 {
            return Color::BLACK;
        }
        return Color::WHITE;
    }

    fn default(iteration_ratio: f32) -> Color {
        Color::RGB(
            (f32::powf(iteration_ratio * 360.0, 1.5) % 255.0) as u8,
            100,
            (iteration_ratio * 255.0) as u8,
        )
    }

    fn lch(iteration_ratio: f32) -> Color {
        let v = 1.0 - f32::powf(f32::cos(PI * iteration_ratio), 2.0);
        let hue = f32::powf(360.0 * iteration_ratio, 1.5) % 360.0;
        let pal = palette::Lch::lch(75.0 - 75.0 * v, 28.0 + 75.0 - 75.0 * v, hue.into());
        ColorAlg::palette_to_sdl_color(pal)
    }

    fn hsv(iteration_ratio: f32) -> Color {
        let hue = f32::powf(iteration_ratio * 360.0, 1.5) % 360.0;
        let color = palette::Hsv::hsv(hue.into(), 100.0, iteration_ratio * 100.0);
        ColorAlg::palette_to_sdl_color(color)
    }

    fn hsl(iteration_ratio: f32) -> Color {
        let hue = f32::powf(iteration_ratio * 360.0, 1.5) % 360.0;
        let color = palette::Hsl::hsl(hue.into(), 50.0, iteration_ratio * 100.0);
        ColorAlg::palette_to_sdl_color(color)
    }
}