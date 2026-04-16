//! Control-window bootstrap and public UI state exports.

mod app;
pub(crate) mod legend;
mod state;
mod theme;
mod validation;

#[allow(unused_imports)]
pub use state::{EqRender, FieldKind, GridUiState, LegendKind, LegendState, SpacialEqs};

use crate::app::ui::app::ControlApp;
use std::sync::{Arc, Mutex};
use std::thread;
use winit::event_loop::EventLoop;
use winit::platform::x11::EventLoopBuilderExtX11;

/// Spawns the control window on a dedicated thread.
///
/// The returned thread owns the `eframe` event loop and only exchanges data with the render loop
/// through the supplied `Arc<Mutex<GridUiState>>`. That mutex is the only shared-state boundary
/// between the UI and the renderer; the UI thread does not touch OpenGL resources or camera
/// state.
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
