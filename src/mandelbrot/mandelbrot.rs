use num::Complex;
use sdl2::rect::{Point, Rect};
use sdl2::pixels::Color;

use crate::mandelbrot::color_algs::ColorAlg;

#[derive(Debug)]
pub struct Mandelbrot {
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    max_iterations: i32,
    width: u32,
    height: u32,
    rendered: bool,
    julia: bool,
    color: ColorAlg,
    mouse_pos: Option<Point>
}

impl Mandelbrot {
    pub fn new(width: u32, height: u32) -> Self {
        Mandelbrot {
            x_min: -2.0,
            x_max: 1.0,
            y_min: -1.0,
            y_max: 1.0,
            max_iterations: 500,
            width,
            height,
            rendered: false,
            julia: false,
            color: ColorAlg::DEFAULT,
            mouse_pos: None
        }
    }

    pub fn x_bounds(&self) -> (f64, f64) {
        (self.x_min, self.x_max)
    }

    pub fn y_bounds(&self) -> (f64, f64) {
        (self.y_min, self.y_max)
    }

    pub fn reset(&mut self) {
        *self = Mandelbrot::new(self.width, self.height);
    }

    pub fn set_max_iter(&mut self, max_iterations: i32) {
        self.max_iterations = max_iterations;
        self.rendered = false;
    }

    pub fn get_max_iter(&self) -> i32{
        self.max_iterations
    }

    pub fn change_max_iter(&mut self, max_iterations_change: i32) {
        self.max_iterations += max_iterations_change;
        self.rendered = false;
    }

    fn zoom(&mut self, scale: f64, new_center: Point) {
        println!("{:?}", self);
        let x_percentage = new_center.x() as f64 / self.width as f64;
        let y_percentage = new_center.y() as f64 / self.height as f64;

        let x_span = self.x_max - self.x_min;
        let y_span = self.y_max - self.y_min;
        let new_x_center = self.x_min + x_span * x_percentage;
        let new_y_center = self.y_min + y_span * y_percentage;

        self.x_min = new_x_center - (x_span * scale * 0.5);
        self.x_max = new_x_center + (x_span * scale * 0.5);
        self.y_min = new_y_center - (y_span * scale * 0.5);
        self.y_max = new_y_center + (y_span * scale * 0.5);
        self.rendered = false;
        println!("{:?}", self);
    }

    pub fn zoom_between_points(&mut self, points: (Point, Point), zoom_out: bool) {
        let rect = Rect::from_enclose_points(&[points.0, points.1], None).unwrap();

        let scale: f64 = match zoom_out {
            false => rect.width() as f64 / self.width as f64,
            true => 2.0 - rect.width() as f64 / self.width as f64,
        };
        self.zoom(scale, rect.center());
    }

    pub fn set_rendered(&mut self, rendered: bool) {
        self.rendered = rendered;
    }

    pub fn is_rendered(&self) ->bool {
        self.rendered
    }

    fn get_complex_point(&self, x: u32, y: u32) -> Complex<f64> {
        let x_percent = x as f64 / self.width as f64;
        let y_percent = y as f64 / self.height as f64;

        let comp_x = self.x_min + (self.x_max - self.x_min) * x_percent;
        let comp_y = self.y_min + (self.y_max - self.y_min) * y_percent;
        Complex::new(comp_x, comp_y)
    }

    fn mandelbrot_point(&self, x: u32, y: u32) -> Color {
        let mut z = Complex { re: 0.0, im: 0.0 };
        let c = self.get_complex_point(x, y);

        for i in 0..=self.max_iterations {
            if z.norm() > 2.0 {
                let iteration_ratio = i as f32 / self.max_iterations as f32;
                return self.color.get_rgb_color(iteration_ratio);
            }
            z = z * z + c;
        }
        return Color::BLACK;
    }

    fn julia_point(&self, x: u32, y: u32) -> Color {
        if let Some(mp) = self.mouse_pos {
            let mut z = self.get_complex_point(x, y);
            let c = self.get_complex_point(mp.x() as u32, mp.y() as u32);

            for i in 0..=self.max_iterations/10 {
                if z.norm() > 2.0 {
                    let iteration_ratio = i as f32 / self.max_iterations as f32;
                    return self.color.get_rgb_color(iteration_ratio);
                }
                z = z * z + c;
            }
        }
        return Color::BLACK;
    }

    pub fn point_color(&self, x: u32, y: u32) -> Color {
        match self.julia {
            true => self.julia_point(x, y),
            false => self.mandelbrot_point(x, y)
        }
    }

    pub fn toggle_julia(&mut self) {
        self.julia = !self.julia;
        self.set_rendered(false);
    }

    pub fn change_color(&mut self) {
        self.color = self.color.next();
        self.set_rendered(false);
    }

    pub fn color(&self) -> ColorAlg {
        self.color
    }

    pub fn set_mouse_pos(&mut self, mouse_pos: Point) {
        self.mouse_pos = Some(mouse_pos);
        if self.julia {
            self.set_rendered(false);
        }
    }
}
