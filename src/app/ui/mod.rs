mod app;
mod state;
mod theme;
mod validation;

#[allow(unused_imports)]
pub use state::{EqRender, GridUiState, SpacialEqs};

use crate::app::ui::app::ControlApp;
use std::sync::{Arc, Mutex};
use std::thread;
use winit::event_loop::EventLoop;
use winit::platform::x11::EventLoopBuilderExtX11;

pub fn spawn_control_window(state: Arc<Mutex<GridUiState>>) {
    thread::spawn(move || {
        let options = eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_inner_size([400.0, 580.0])
                .with_title("Grid Controls"),
            renderer: eframe::Renderer::Glow,
            ..Default::default()
        };

        let eventloop = EventLoop::with_user_event()
            .with_any_thread(true)
            .build()
            .unwrap();

        let shared = state.clone();
        let mut app = eframe::create_native(
            "Grid Controls",
            options,
            Box::new(move |_cc| Ok(Box::new(ControlApp::new(shared.clone())))),
            &eventloop,
        );
        eventloop
            .run_app(&mut app)
            .expect("Unable to run eframe app");
    });
}
