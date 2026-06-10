//! egui control panel for editing the grid, field, and tangent-view settings.

mod tabs;

use crate::app::ui::legend::show_legend_window;
use crate::app::ui::presets::{EmPreset, FieldPreset, GridPreset};
use crate::app::ui::state::{ControlTab, FieldKind, GridUiState};
use crate::app::ui::theme::{
    self, ACCENT, BORDER, CRAYOLA_BLUE, JET_BLACK, MUTED, PANEL, RASPBERRY, SHADOW_GREY, TEXT,
};
use crate::app::ui::validation::validate_ui_state;
use eframe::egui::{self, Color32, Stroke};
use eframe::epaint::{CornerRadius, Margin};
use std::sync::{Arc, Mutex};

pub(super) trait PresetLabel {
    fn label(self) -> &'static str;
}

impl PresetLabel for GridPreset {
    fn label(self) -> &'static str {
        self.label
    }
}

impl PresetLabel for FieldPreset {
    fn label(self) -> &'static str {
        self.label
    }
}

impl PresetLabel for EmPreset {
    fn label(self) -> &'static str {
        self.label
    }
}

pub(crate) struct ControlApp {
    state: Arc<Mutex<GridUiState>>,
    styled: bool,
    error_popup: Option<String>,
    active_tab: ControlTab,
}

impl ControlApp {
    /// Creates the egui control application around the shared UI state.
    pub(crate) fn new(state: Arc<Mutex<GridUiState>>) -> Self {
        Self {
            state,
            styled: false,
            error_popup: None,
            active_tab: ControlTab::Grid,
        }
    }

    /// Renders the grid or field tab selector.
    fn render_tab_bar(ui: &mut egui::Ui, active_tab: &mut ControlTab) {
        egui::Frame::new()
            .fill(SHADOW_GREY)
            .inner_margin(Margin::same(4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if Self::tab_button(ui, *active_tab == ControlTab::Grid, "Grid") {
                        *active_tab = ControlTab::Grid;
                    }
                    if Self::tab_button(ui, *active_tab == ControlTab::Field, "Field") {
                        *active_tab = ControlTab::Field;
                    }
                    if Self::tab_button(ui, *active_tab == ControlTab::Em, "EM") {
                        *active_tab = ControlTab::Em;
                    }
                });
            });
    }

    fn compact_button(ui: &mut egui::Ui, label: &str) -> bool {
        ui.add(
            egui::Button::new(egui::RichText::new(label).color(TEXT))
                .fill(JET_BLACK)
                .stroke(Stroke::new(1.0, BORDER))
                .min_size(egui::vec2(116.0, 28.0))
                .corner_radius(CornerRadius::same(6)),
        )
        .clicked()
    }

    /// Validates the current UI state and bumps the apply counter on success.
    fn handle_apply(data: &mut GridUiState, error_popup: &mut Option<String>) {
        match validate_ui_state(data) {
            Ok(validated) => {
                data.coords_sys = validated.coords_sys;
                data.scalar_field = validated.scalar_field;
                data.field = validated.field;
                data.em = validated.em;
                data.apply_counter += 1;
            }
            Err(error) => *error_popup = Some(error),
        }
    }
    /// Renders the Apply button and runs validation when it is clicked.
    fn render_apply_button(
        ui: &mut egui::Ui,
        data: &mut GridUiState,
        error_popup: &mut Option<String>,
    ) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let clicked = ui
                .add(
                    egui::Button::new(egui::RichText::new("Apply").color(Color32::WHITE))
                        .fill(ACCENT)
                        .min_size(egui::vec2(120.0, 34.0))
                        .corner_radius(CornerRadius::same(6)),
                )
                .clicked();

            if clicked {
                Self::handle_apply(data, error_popup);
            }
        });
    }

    /// Shows the modal popup used to explain equation-validation failures.
    fn show_error_popup(&mut self, ctx: &egui::Context) {
        let Some(message) = self.error_popup.as_ref() else {
            return;
        };

        let mut open = true;
        let mut close_now = false;
        egui::Window::new("Input error")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .frame(
                egui::Frame::window(&ctx.global_style())
                    .fill(PANEL)
                    .stroke(Stroke::new(1.5, BORDER))
                    .inner_margin(Margin::same(14)),
            )
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("Please fix your equations")
                        .color(TEXT)
                        .strong(),
                );
                ui.add_space(6.0);
                ui.label(egui::RichText::new(message).color(MUTED));
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Tips:").color(TEXT).strong());
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new("Don't forget to use * for multiplication.").color(MUTED),
                );
                ui.add_space(10.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new("Close").color(Color32::WHITE))
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

    /// Renders one tab button and returns whether it was clicked.
    fn tab_button(ui: &mut egui::Ui, selected: bool, label: &str) -> bool {
        let fill = if selected { ACCENT } else { JET_BLACK };
        let stroke = if selected {
            Stroke::new(1.0, ACCENT)
        } else {
            Stroke::new(1.0, BORDER)
        };

        ui.add(
            egui::Button::new(egui::RichText::new(label).color(TEXT).strong())
                .fill(fill)
                .stroke(stroke)
                .min_size(egui::vec2(96.0, 30.0))
                .corner_radius(CornerRadius::same(6)),
        )
        .clicked()
    }

    /// Renders the scalar/vector selector for the editable field input.
    fn render_field_kind_selector(ui: &mut egui::Ui, field_kind: &mut FieldKind) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Input type").color(TEXT));
            if Self::tab_button(ui, *field_kind == FieldKind::Scalar, "Scalar") {
                *field_kind = FieldKind::Scalar;
            }
            if Self::tab_button(ui, *field_kind == FieldKind::Vector, "Vector") {
                *field_kind = FieldKind::Vector;
            }
        });
    }

    /// Renders the density slider for one grid axis.
    fn density_slider(ui: &mut egui::Ui, value: &mut f64, label: &str) {
        ui.add(
            egui::Slider::new(value, 0.0..=20.0)
                .logarithmic(false)
                .text(label)
                .trailing_fill(true),
        );
    }

    /// Renders one labeled equation input row.
    fn eq_row(ui: &mut egui::Ui, label: &str, value: &mut String) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(label).color(TEXT));
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

    /// Renders a hover-only help marker for compact UI guidance.
    pub(super) fn help_dot(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
        draw_info_dot(ui).on_hover_ui(add_contents);
    }

    /// Renders the min and max editors for one axis bound pair.
    fn bounds_row(ui: &mut egui::Ui, label: &str, bounds: &mut (f64, f64)) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(label).color(TEXT));
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

    /// Renders the main control window contents against the shared state.
    fn render_ui(&mut self, ui: &mut egui::Ui) {
        let active_tab = &mut self.active_tab;
        let error_popup = &mut self.error_popup;
        let state = self.state.clone();

        let mut data = state.lock().expect("UI state poisoned");
        Self::render_tab_bar(ui, active_tab);
        ui.add_space(16.0);
        Self::render_active_tab(ui, *active_tab, &mut data);
        ui.add_space(10.0);
        Self::render_apply_button(ui, &mut data, error_popup);
    }
}

impl eframe::App for ControlApp {
    /// Applies the shared theme and keeps the detached legend window synchronized.
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.styled {
            theme::apply_style(ctx);
            self.styled = true;
        }
        let legend = self.state.lock().expect("UI state poisoned").legend;
        show_legend_window(ctx, legend);
    }

    /// Renders the central control panel and any active error popup.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Frame::central_panel(ui.style()).show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    self.render_ui(ui);
                });
        });
        self.show_error_popup(ui.ctx());
    }

    /// Requests periodic repainting so the control window stays responsive.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(33));
    }
}

/// Renders the small help dot shown beside equation inputs.
fn info_dot(ui: &mut egui::Ui) -> egui::Response {
    draw_info_dot(ui).on_hover_ui(equation_help_popup)
}

fn draw_info_dot(ui: &mut egui::Ui) -> egui::Response {
    ui.add_space(4.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
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
    response
}

fn equation_help_popup(ui: &mut egui::Ui) {
    ui.set_max_width(320.0);
    ui.label(egui::RichText::new("Equation help").color(TEXT).strong());
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("Use x, y, and z for spatial coordinates. EM equations also allow t.")
            .color(MUTED),
    );
    ui.label(
        egui::RichText::new("Write explicit multiplication: x*y, 2*sin(z), not xy.").color(MUTED),
    );
    ui.label(
        egui::RichText::new("Common functions include sin, cos, tan, sqrt, exp, ln, log, and abs.")
            .color(MUTED),
    );
}
