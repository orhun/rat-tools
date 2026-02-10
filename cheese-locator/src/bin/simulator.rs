use embedded_graphics_simulator::sdl2::Keycode;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use mousefood::embedded_graphics::geometry;
use mousefood::embedded_graphics::pixelcolor::Rgb565;
use mousefood::error::Error;
use mousefood::prelude::*;
use ratatui::Terminal;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

#[path = "../app.rs"]
mod app;
use app::App;

fn main() -> Result<(), Error> {
    let output_settings = OutputSettingsBuilder::new().scale(3).build();
    let simulator_window = Rc::new(RefCell::new(Window::new(
        "mousefood simulator",
        &output_settings,
    )));
    simulator_window.borrow_mut().set_max_fps(30);

    let mut display = SimulatorDisplay::<Rgb565>::new(geometry::Size::new(128, 64));

    let window_handle = Rc::clone(&simulator_window);
    let backend_config = EmbeddedBackendConfig {
        flush_callback: Box::new(move |display| {
            window_handle.borrow_mut().update(display);
        }),
        ..Default::default()
    };
    let backend: EmbeddedBackend<SimulatorDisplay<_>, _> =
        EmbeddedBackend::new(&mut display, backend_config);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    let start = Instant::now();
    let mut button_pressed = false;
    let mut menu_pressed = false;
    let mut step_pulse = false;

    loop {
        terminal.draw(|f| {
            app.render(f);
        })?;

        let now_ms = start.elapsed().as_millis() as u64;
        let accel = if step_pulse {
            step_pulse = false;
            Some((2.0, 0.0, 0.0))
        } else {
            Some((0.0, 0.0, 1.0))
        };
        app.tick(now_ms, button_pressed, menu_pressed, accel);

        let window = simulator_window.borrow_mut();
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => return Ok(()),
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Left => button_pressed = true,
                    Keycode::Right => menu_pressed = true,
                    Keycode::Up => step_pulse = true,
                    _ => {}
                },
                SimulatorEvent::KeyUp { keycode, .. } => match keycode {
                    Keycode::Left => button_pressed = false,
                    Keycode::Right => menu_pressed = false,
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
