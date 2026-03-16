use std::sync::Arc;

use femtovg::{Canvas, Color, FillRule, Paint, Path};
use instant::Instant;
use winit::{event::WindowEvent, window::Window};

mod helpers;
use helpers::WindowSurface;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    helpers::start(800, 600, "Depth Buffer - Crossing Planes", false);
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
                canvas.clear_rect(0, 0, size.width, size.height, Color::rgbf(0.15, 0.15, 0.2));

                let elapsed = start.elapsed().as_secs_f32();
                let width = size.width as f32;
                let height = size.height as f32;
                let center = [width / 2.0, height / 2.0];

                let tilt = (elapsed * 0.5).sin() * 0.4;

                let mut plane1 = Path::new();
                plane1.rect([center[0] - 200.0, center[1] - 100.0], [400.0, 200.0]);

                let mut plane2 = Path::new();
                plane2.rect([center[0] - 100.0, center[1] - 150.0], [200.0, 300.0]);

                canvas.fill_path_with_depth(
                    &plane1,
                    &Paint::color(Color::rgba(200, 60, 60, 255)),
                    FillRule::default(),
                    |x, _y| {
                        let normalized = (x - (center[0] - 200.0)) / 400.0;
                        0.3 + (normalized - 0.5) * tilt
                    },
                );

                canvas.fill_path_with_depth(
                    &plane2,
                    &Paint::color(Color::rgba(60, 60, 200, 255)),
                    FillRule::default(),
                    |_x, y| {
                        let normalized = (y - (center[1] - 150.0)) / 300.0;
                        0.3 + (normalized - 0.5) * -tilt
                    },
                );

                canvas.set_depth(0.0);
                surface.present(&mut canvas);
                window.request_redraw();
            }
            _ => (),
        }),
        device_event: None,
    }
}
