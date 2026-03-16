use std::{f32::consts::PI, sync::Arc};

use femtovg::{Canvas, Color, FillRule, Paint, Path, Renderer, StrokeSettings};
use instant::Instant;
use winit::{event::WindowEvent, window::Window};

mod helpers;
use helpers::WindowSurface;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    helpers::start(1000, 750, "Ocean with Depth Buffer", false);
    #[cfg(target_arch = "wasm32")]
    helpers::start();
}

fn run<W: WindowSurface + 'static>(
    mut canvas: Canvas<W::Renderer>,
    mut surface: W,
    window: Arc<Window>,
) -> helpers::Callbacks {
    let start = Instant::now();

    helpers::Callbacks {
        window_event: Box::new(move |event, event_loop| match event {
            #[cfg(not(target_arch = "wasm32"))]
            WindowEvent::Resized(physical_size) => {
                surface.resize(physical_size.width, physical_size.height);
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let size = window.inner_size();
                let dpi_factor = window.scale_factor();
                canvas.set_size(size.width, size.height, dpi_factor as f32);

                let width = size.width as f32;
                let height = size.height as f32;

                canvas.clear_rect(0, 0, size.width, size.height, Color::rgbf(0.6, 0.85, 1.0));

                let elapsed = start.elapsed().as_secs_f32();

                draw_ocean(&mut canvas, elapsed, width, height);

                surface.present(&mut canvas);
                window.request_redraw();
            }
            _ => (),
        }),
        device_event: None,
    }
}

fn wave_height(x: f32, y: f32, time: f32) -> f32 {
    (x * 0.02 + time * 1.5).sin() * 12.0
        + (y * 0.03 + time * 0.8).sin() * 8.0
        + (x * 0.01 + y * 0.015 + time * 0.5).sin() * 15.0
}

fn draw_ocean<T: Renderer>(canvas: &mut Canvas<T>, time: f32, width: f32, height: f32) {
    let horizon = height * 0.35;
    let strips = 80;
    let cols = 120;

    for i in (0..strips).rev() {
        let t0 = i as f32 / strips as f32;
        let t1 = (i + 1) as f32 / strips as f32;

        let y0 = horizon + t0 * (height - horizon);
        let y1 = horizon + t1 * (height - horizon);

        let perspective0 = 0.2 + t0 * 0.8;
        let perspective1 = 0.2 + t1 * 0.8;

        let world_z0 = 1.0 / (t0 + 0.01);
        let world_z1 = 1.0 / (t1 + 0.01);

        let depth = 1.0 - t0;
        canvas.set_depth(depth);

        let water_dark = 0.15 + t0 * 0.25;
        let water_green = 0.3 + t0 * 0.3;
        let water_blue = 0.5 + t0 * 0.4;
        let water_alpha = 0.7 + t0 * 0.3;

        let mut strip = Path::new();

        strip.move_to([0.0, y0 + wave_height(0.0, world_z0, time) * perspective0]);
        for j in 1..=cols {
            let x = j as f32 / cols as f32 * width;
            let wave = wave_height(x, world_z0, time) * perspective0;
            strip.line_to([x, y0 + wave]);
        }

        for j in (0..=cols).rev() {
            let x = j as f32 / cols as f32 * width;
            let wave = wave_height(x, world_z1, time) * perspective1;
            strip.line_to([x, y1 + wave]);
        }
        strip.close();

        let color = Color::rgbaf(water_dark, water_green, water_blue, water_alpha);
        canvas.fill_path(&strip, &Paint::color(color), FillRule::default());

        if i % 4 == 0 {
            let foam_alpha = (0.3 - t0).max(0.0) * 2.0;
            if foam_alpha > 0.01 {
                let mut foam = Path::new();
                let foam_y = y0 + wave_height(0.0, world_z0, time) * perspective0;
                foam.move_to([0.0, foam_y]);
                for j in 1..=cols {
                    let x = j as f32 / cols as f32 * width;
                    let wave = wave_height(x, world_z0, time) * perspective0;
                    foam.line_to([x, y0 + wave]);
                }
                let stroke = StrokeSettings::new(1.5 * perspective0);
                canvas.stroke_path(&foam, &Paint::color(Color::rgbaf(1.0, 1.0, 1.0, foam_alpha)), &stroke);
            }
        }
    }

    let beach_depth = 0.0;
    canvas.set_depth(beach_depth);

    let sand_strips = 20;
    for i in 0..sand_strips {
        let t = i as f32 / sand_strips as f32;
        let y_start = height * 0.7 + t * height * 0.3;
        let y_end = height * 0.7 + (t + 1.0 / sand_strips as f32) * height * 0.3;

        let slope_depth = t * 0.5;
        canvas.set_depth(slope_depth);

        let sand_r = 0.76 + t * 0.1;
        let sand_g = 0.7 + t * 0.08;
        let sand_b = 0.5 + t * 0.05;

        let wet_factor = (1.0 - t * 3.0).max(0.0);
        let wave_edge = wave_height(width * 0.5, 1.0 / (0.7 + 0.01), time) * 0.5;
        let wet = if y_start < height * 0.75 + wave_edge {
            wet_factor * 0.3
        } else {
            0.0
        };

        let mut sand = Path::new();
        sand.rect([0.0, y_start], [width, y_end - y_start]);
        canvas.fill_path(
            &sand,
            &Paint::color(Color::rgbf(sand_r - wet, sand_g - wet, sand_b - wet)),
            FillRule::default(),
        );
    }

    canvas.set_depth(0.0);
}
