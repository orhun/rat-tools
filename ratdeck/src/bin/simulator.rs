use embedded_graphics_simulator::sdl2::Keycode;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use mousefood::embedded_graphics::geometry;
use mousefood::embedded_graphics::pixelcolor::Rgb565;
use mousefood::error::Error;
use mousefood::{fonts, prelude::*};
use ratatui::Terminal;
use ratdeck::app::App;
use std::cell::RefCell;
use std::rc::Rc;

fn main() -> Result<(), Error> {
    let output_settings = OutputSettingsBuilder::new().scale(3).build();
    let simulator_window = Rc::new(RefCell::new(Window::new(
        "mousefood simulator",
        &output_settings,
    )));
    simulator_window.borrow_mut().set_max_fps(30);

    let mut display = SimulatorDisplay::<Rgb565>::new(geometry::Size::new(320, 240));

    let window_handle = Rc::clone(&simulator_window);
    let backend_config = EmbeddedBackendConfig {
        flush_callback: Box::new(move |display| {
            window_handle.borrow_mut().update(display);
        }),
        font_regular: fonts::MONO_8X13,
        font_bold: Some(fonts::MONO_8X13_BOLD),
        font_italic: Some(fonts::MONO_8X13_ITALIC),
        ..Default::default()
    };
    let backend: EmbeddedBackend<SimulatorDisplay<_>, _> =
        EmbeddedBackend::new(&mut display, backend_config);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| {
            app.render(f);
        })?;

        app.render_image(terminal.backend_mut().display_mut());

        let window = simulator_window.borrow_mut();
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => return Ok(()),
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Right | Keycode::Down => app.next_slide(),
                    Keycode::Left | Keycode::Up => app.prev_slide(),
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
