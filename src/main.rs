extern crate sdl2;

use rayon::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;
use std::time::{Duration, SystemTime};

mod mandelbrot;
use crate::mandelbrot::mandelbrot::Mandelbrot;

fn create_text_texture<'a>(
    texture_creator: &'a TextureCreator<WindowContext>,
    text: &str,
    font: &sdl2::ttf::Font,
    color: Color,
) -> Result<(Texture<'a>, (u32, u32)), String> {
    let surface = font
        .render(text)
        .blended(color)
        .map_err(|e| e.to_string())?;
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;
    Ok((texture, surface.size()))
}

fn main() -> Result<(), String> {
    const WIDTH: u32 = 1366;
    const HEIGHT: u32 = 768;
    const WINDOW_NAME: &str = "Mandelbrot Explorer";

    let mut mand = Mandelbrot::new(WIDTH, HEIGHT);

    let sdl = sdl2::init().map_err(|e| e.to_string())?;

    let sdl_video = sdl.video().map_err(|e| e.to_string())?;

    let window = sdl_video
        .window(WINDOW_NAME, WIDTH, HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .build()
        .map_err(|e| e.to_string())?;

    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl.event_pump().map_err(|e| e.to_string())?;

    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let font = ttf_context.load_font("/usr/share/fonts/TTF/Inconsolata-Regular.ttf", 20)?;

    let mut mandelbrot_texture = texture_creator
        .create_texture_target(None, WIDTH, HEIGHT)
        .map_err(|e| e.to_string())?;

    canvas.present();

    let coords_vec: Vec<(i32, i32)> = (0..WIDTH)
        .flat_map(|x| (0..HEIGHT).map(move |y| (y as i32, x as i32)))
        .collect();

    let mut down_pos: Option<Point> = None;
    let mut mouse_box: Option<Rect> = None;
    let mut show_info = false;
    'paint: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown { keycode, .. } => match keycode {
                    Some(Keycode::Q) => {
                        let change = std::cmp::max(2, mand.get_max_iter() / 10);
                        mand.change_max_iter(change);
                    }
                    Some(Keycode::A) => {
                        let change = -std::cmp::max(2, mand.get_max_iter() / 10);
                        mand.change_max_iter(change)
                    }
                    Some(Keycode::L) => mand.set_max_iter(5),
                    Some(Keycode::J) => mand.toggle_julia(),
                    Some(Keycode::R) => mand.reset(),
                    Some(Keycode::C) => mand.change_color(),
                    Some(Keycode::I) => show_info = !show_info,
                    Some(Keycode::Escape) => break 'paint,
                    Some(_) | None => {}
                },
                Event::Quit { .. } => break 'paint,
                Event::MouseButtonDown {
                    mouse_btn: button,
                    x: mouse_x,
                    y: mouse_y,
                    ..
                } => {
                    println!("DOWN {:?} ({}, {})", button, mouse_y, mouse_x);
                    match button {
                        MouseButton::Left | MouseButton::Right => {
                            down_pos = Some(Point::new(mouse_x, mouse_y));
                        }
                        _ => {}
                    }
                }
                Event::MouseButtonUp {
                    mouse_btn: button,
                    x: mouse_x,
                    y: mouse_y,
                    ..
                } => {
                    println!("UP {:?} ({}, {})", button, mouse_y, mouse_x);
                    if button != MouseButton::Left && button != MouseButton::Right
                        || down_pos.is_none()
                    {
                        continue;
                    }

                    let up_pos = Point::new(mouse_x, mouse_y);

                    let zoom_out = button == MouseButton::Right;
                    mand.zoom_between_points((down_pos.unwrap(), up_pos), zoom_out);
                    down_pos = None;
                    mouse_box = None;
                }
                Event::MouseMotion {
                    x: mouse_x,
                    y: mouse_y,
                    ..
                } => {
                    let mouse_pos = Point::new(mouse_x, mouse_y);
                    mand.set_mouse_pos(mouse_pos);
                    if let Some(down_pos) = down_pos {
                        let points = &[down_pos, mouse_pos];
                        mouse_box = Some(Rect::from_enclose_points(points, None).unwrap());
                    }
                }
                _ => {}
            }
        }

        let mut now = SystemTime::now();

        // Recalculate mandelbrot
        if !mand.is_rendered() {
            let pixels: Vec<_> = coords_vec
                .par_iter()
                .map(|(y, x)| {
                    (
                        Point::new(*x, *y),
                        mand.point_color(x.clone() as u32, y.clone() as u32),
                    )
                })
                .collect();

            println!("Post calculate {}", now.elapsed().unwrap().as_millis());
            now = SystemTime::now();

            canvas
                .with_texture_canvas(&mut mandelbrot_texture, |texture_canvas| {
                    texture_canvas.clear();
                    for (point, color) in pixels {
                        texture_canvas.set_draw_color(color);
                        texture_canvas.draw_point(point).unwrap();
                    }
                })
                .unwrap();

            mand.set_rendered(true);

            println!(
                "Post texture painting {}",
                now.elapsed().unwrap().as_millis()
            );
        }

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas
            .copy(
                &mandelbrot_texture,
                None,
                Some(Rect::new(0, 0, WIDTH, HEIGHT)),
            )
            .unwrap();

        if let Some(zoom_box) = mouse_box {
            canvas.set_draw_color(Color::RED);
            canvas.draw_rect(zoom_box)?;
            canvas.set_draw_color(Color::RGBA(0, 0, 0, 50));
            canvas.fill_rect(zoom_box)?;
        }

        if show_info {
            let (x_min, x_max) = mand.x_bounds();
            let (y_min, y_max) = mand.y_bounds();
            let text_textures: Vec<(Texture, (u32, u32))> = [
                format!("X {} -> {}", x_min, x_max),
                format!("Y {} -> {}", y_min, y_max),
                format!("Iters: {}, Color: {:?}", mand.get_max_iter(), mand.color()),
            ]
            .iter()
            .map(|text| create_text_texture(&texture_creator, &text, &font, Color::WHITE).unwrap())
            .collect();

            const PADDING: u32 = 5;

            let max_width: u32 = text_textures.iter().map(|e| e.1.0).max().unwrap();
            let text_height: u32 = text_textures[0].1.1;

            canvas.set_draw_color(Color::RGBA(0, 0, 0, 150));
            let background_rect = Rect::new(
                0,
                0,
                max_width + PADDING * 2,
                text_height * text_textures.len() as u32 + PADDING * 2,
            );
            canvas.fill_rect(Some(background_rect))?;

            for (i, (text_texture, (width, height))) in text_textures.iter().enumerate() {
                let height_offset = (PADDING + text_height * i as u32) as i32;
                let dest = Some(Rect::new(PADDING as i32, height_offset, *width, *height));
                canvas.copy(&text_texture, None, dest)?;
            }
        }

        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000u32));
    }
    Ok(())
}
