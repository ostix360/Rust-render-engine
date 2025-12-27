
use eframe::egui::{self, Color32, RichText, Stroke};
use eframe::epaint::{CornerRadius, Margin};
use std::sync::{Arc, Mutex};
use std::thread;
use exmex::{parse, Express};
use winit::event_loop::EventLoop;
use winit::platform::x11::EventLoopBuilderExtX11;
use crate::maths::Expr;

const RASPBERRY: Color32 = Color32::from_rgb(0xB0, 0x18, 0xA2);
const CRAYOLA_BLUE: Color32 = Color32::from_rgb(0x3A, 0x74, 0xE0);
const GOLDEN_GLOW: Color32 = Color32::from_rgb(0xD1, 0xCD, 0x07);
const OLIVE_LEAF: Color32 = Color32::from_rgb(0x61, 0x60, 0x36);
const DARK_KHAKI: Color32 = Color32::from_rgb(0x48, 0x45, 0x2F);
const SHADOW_GREY: Color32 = Color32::from_rgb(0x1E, 0x17, 0x22);
const INK_BLACK: Color32 = Color32::from_rgb(0x08, 0x10, 0x1F);
const JET_BLACK: Color32 = Color32::from_rgb(0x1E, 0x26, 0x33);

const PANEL: Color32 = INK_BLACK;
const BORDER: Color32 = DARK_KHAKI;
const ACCENT: Color32 = CRAYOLA_BLUE;
const TEXT: Color32 = Color32::from_rgb(0xE9, 0xEF, 0xFE);
const MUTED: Color32 = Color32::from_rgb(175, 182, 195);

#[derive(Debug, Clone)]
pub struct GridUiState {
    pub render_3d: bool,
    pub eq_x: String,
    pub eq_y: String,
    pub eq_z: String,
    pub density_x: f32,
    pub density_y: f32,
    pub density_z: f32,
    pub bounds_x: (f32, f32),
    pub bounds_y: (f32, f32),
    pub bounds_z: (f32, f32),
}

impl Default for GridUiState {
    fn default() -> Self {
        Self {
            render_3d: true,
            eq_x: "x*cos(y) * sin(z)".to_string(),
            eq_y: "x*sin(y) * sin(z)".to_string(),
            eq_z: "x * cos(z)".to_string(),
            density_x: 5.0,
            density_y: 5.0,
            density_z: 5.0,
            bounds_x: (-1.6, 15.0),
            bounds_y: (0.0, 7.0),
            bounds_z: (0.0, 7.0),
        }
    }
}

pub fn spawn_control_window(state: Arc<Mutex<GridUiState>>) {
    thread::spawn(move || {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([400.0, 580.0])
                .with_title("Grid Controls"),
            renderer: eframe::Renderer::Glow,
            ..Default::default()
        };

        let eventloop = EventLoop::with_user_event().with_any_thread(true).build().unwrap();

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

struct ControlApp {
    state: Arc<Mutex<GridUiState>>,
    styled: bool,
    error_popup: Option<String>,
}

impl ControlApp {
    fn new(state: Arc<Mutex<GridUiState>>) -> Self {
        Self {
            state,
            styled: false,
            error_popup: None,
        }
    }

    fn apply_style(&mut self, ctx: &egui::Context) {
        ctx.set_visuals(egui::Visuals::dark());
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.window_margin = Margin::same(18);

        style.visuals.dark_mode = true;
        style.visuals.override_text_color = Some(TEXT);
        style.visuals.panel_fill = PANEL;
        style.visuals.window_fill = INK_BLACK;
        style.visuals.window_stroke = Stroke::new(1.0, BORDER);
        style.visuals.window_corner_radius = CornerRadius::same(8);
        style.visuals.widgets.noninteractive.bg_fill = PANEL;
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
        style.visuals.widgets.inactive.bg_fill = PANEL;
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
        style.visuals.widgets.inactive.weak_bg_fill = JET_BLACK;
        style.visuals.widgets.inactive.corner_radius = CornerRadius::same(4);
        style.visuals.widgets.hovered.corner_radius = CornerRadius::same(4);
        style.visuals.widgets.active.corner_radius = CornerRadius::same(4);
        style.visuals.widgets.open.corner_radius = CornerRadius::same(4);
        style.visuals.widgets.active.fg_stroke = Stroke::new(2.0, TEXT);
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, TEXT);
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, MUTED);
        style.visuals.text_edit_bg_color = Some(JET_BLACK);
        style.visuals.selection.bg_fill = ACCENT;
        style.visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(30, 60, 120));
        style.visuals.slider_trailing_fill = true;
        ctx.set_style(style);
    }

    fn section_heading(text: &str) -> RichText {
        RichText::new(text).strong()
    }

    fn eq_row(ui: &mut egui::Ui, label: &str, value: &mut String) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(label).color(TEXT));
            ui.add(
                egui::TextEdit::singleline(value)
                    .desired_width(220.0)
                    .hint_text("Enter expression")
                    .text_color(TEXT),
            );
            ui.add_space(10.0);
            info_dot(ui);
        });
    }

    fn bounds_row(ui: &mut egui::Ui, label: &str, bounds: &mut (f32, f32)) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(label).color(TEXT));
            ui.add(
                egui::DragValue::new(&mut bounds.0)
                    .speed(0.1)
                    .range(-100.0..=100.0),
            );
            ui.label("to");
            ui.add(
                egui::DragValue::new(&mut bounds.1)
                    .speed(0.1)
                    .range(-100.0..=100.0),
            );
        });
    }
}

impl eframe::App for ControlApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.styled {
            self.apply_style(ctx);
            self.styled = true;
        }

        let mut data = self.state.lock().expect("UI state poisoned");

        egui::CentralPanel::default().show(ctx, |ui| {
            // Title bar
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Grid")
                        .color(TEXT)
                        .size(22.0)
                        .strong(),
                );
            });
            ui.add_space(10.0);

            egui::CollapsingHeader::new(Self::section_heading("Coordinate system"))
                .default_open(true)
                .show(ui, |ui| {
                    ui.checkbox(&mut data.render_3d, RichText::new("Render 3D").color(TEXT));
                    ui.separator();
                    Self::eq_row(ui, "Equation x:  x =", &mut data.eq_x);
                    Self::eq_row(ui, "Equation y:  y =", &mut data.eq_y);
                    Self::eq_row(ui, "Equation z:  z =", &mut data.eq_z);
                });

            ui.add_space(8.0);
            egui::CollapsingHeader::new(Self::section_heading("Grid settings"))
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(RichText::new("Line density").color(MUTED));
                    ui.add(
                        egui::Slider::new(&mut data.density_x, 0.1..=10.0)
                            .logarithmic(true)
                            .text("x")
                            .trailing_fill(true),
                    );
                    ui.add(
                        egui::Slider::new(&mut data.density_y, 0.1..=10.0)
                            .logarithmic(true)
                            .text("y")
                            .trailing_fill(true),
                    );
                    ui.add(
                        egui::Slider::new(&mut data.density_z, 0.1..=10.0)
                            .logarithmic(true)
                            .text("z")
                            .trailing_fill(true),
                    );

                    ui.separator();
                    ui.label(RichText::new("Bounds").color(MUTED));
                    ui.horizontal(|ui| {
                        ui.add_space(80.0);
                        ui.colored_label(MUTED, "min");
                        ui.add_space(10.0);
                        ui.colored_label(MUTED, "max");
                    });
                    Self::bounds_row(ui, "Bounds (x):", &mut data.bounds_x);
                    Self::bounds_row(ui, "Bounds (y):", &mut data.bounds_y);
                    Self::bounds_row(ui, "Bounds (z):", &mut data.bounds_z);
                });

            ui.add_space(10.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let apply_pressed = ui
                    .add(
                        egui::Button::new(RichText::new("Apply").color(Color32::WHITE))
                            .fill(ACCENT)
                            .min_size(egui::vec2(120.0, 34.0))
                            .corner_radius(CornerRadius::same(6)),
                    )
                    .clicked();
                if apply_pressed {
                    let res_x = check_eq_validity(&data.eq_x);
                    let res_y = check_eq_validity(&data.eq_y);
                    let res_z = check_eq_validity(&data.eq_z);
                    if res_x.is_ok() && res_y.is_ok() && res_z.is_ok() {
                        // apply changes to the grid and reinit it
                    }else {
                        let msg = format!("Error in equation(s): \
                        \nEquation x: {} \nEquation y: {}\nEquation z: {}",
                                          res_x.err().unwrap_or("No Error here".to_string()),
                                          res_y.err().unwrap_or("No Error here".to_string()),
                                          res_z.err().unwrap_or("No Error here".to_string()));
                        self.error_popup = Some(msg);
                    }
                }
            });
        });

        if let Some(msg) = &mut self.error_popup {
            let mut open = true;
            let mut close_now = false;
            egui::Window::new("Input error")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .frame(
                    egui::Frame::window(&ctx.style())
                        .fill(PANEL)
                        .stroke(Stroke::new(1.5, BORDER))
                        .inner_margin(Margin::same(14)),
                )
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label(RichText::new("Please fix your equations").color(TEXT).strong());
                    ui.add_space(6.0);
                    ui.label(RichText::new(msg.clone()).color(MUTED));
                    ui.add_space(8.0);
                    ui.label(RichText::new("Tips:").color(TEXT).strong());
                    ui.add_space(6.0);
                    ui.label(RichText::new("Don't forget to use * for multiplication.").color(MUTED));
                    ui.add_space(10.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(RichText::new("Close").color(Color32::WHITE))
                                    .fill(RASPBERRY)
                                    .min_size(egui::vec2(90.0, 30.0))
                                    .corner_radius(CornerRadius::same(6)),
                            )
                            .clicked()
                        {
                            close_now = true;
                        }
                    });
                });

            if !open || close_now {
                self.error_popup = None;
            }
        }
    }
}

fn check_eq_validity(eq: &String) -> Result<Expr, String> {
    if eq.is_empty() {
        return Err("Equation cannot be empty".to_string());
    }
    let formal_eq = parse::<f64>(eq).map_err(|e| format!("Invalid equation: {}", e))?;
    let vars = formal_eq.var_names();
    for var in vars.iter() {
        if var != "x" && var != "y" && var != "z" {
            return Err(format!("Invalid variable '{}' in equation. Only 'x', 'y', and 'z' are allowed.", var));
        }
    }
    Ok(formal_eq)
}

fn info_dot(ui: &mut egui::Ui) {
    ui.add_space(4.0);
    let (rect, _response) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
    let painter = ui.painter();
    let center = rect.center();
    painter.circle_filled(center, 8.0, CRAYOLA_BLUE);
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        "?",
        egui::FontId::proportional(10.0),
        Color32::BLACK,
    );
}
