//! Detached legend window and shared color ramp for sampled scalar or dual-form values.

use crate::app::ui::state::{LegendKind, LegendState};
use crate::app::ui::theme::{self, BORDER, MUTED, SHADOW_GREY, TEXT};
use eframe::egui::{self, Color32, Stroke, ViewportBuilder, ViewportCommand, ViewportId};
use eframe::epaint::{CornerRadius, Margin};
use nalgebra::{vector, Vector4};

/// Shows or closes the detached legend viewport for sampled value rendering.
pub(crate) fn show_legend_window(ctx: &egui::Context, legend: Option<LegendState>) {
    let Some(legend) = legend else {
        ctx.send_viewport_cmd_to(legend_viewport_id(), ViewportCommand::Close);
        return;
    };

    let mut builder = ViewportBuilder::default()
        .with_title(window_title(legend.kind))
        .with_inner_size([300.0, 180.0])
        .with_resizable(false);
    if let Some(rect) = ctx.input(|input| input.viewport().outer_rect) {
        builder = builder.with_position([rect.max.x + 14.0, rect.min.y]);
    }

    ctx.show_viewport_deferred(legend_viewport_id(), builder, move |ui, _class| {
        theme::apply_style(ui.ctx());
        egui::CentralPanel::default().show_inside(ui, |ui| {
            render_legend(ui, legend);
        });
    });
}

/// Renders the legend contents inside the detached viewport.
fn render_legend(ui: &mut egui::Ui, legend: LegendState) {
    egui::Frame::new()
        .fill(SHADOW_GREY)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(title(legend.kind)).color(TEXT).strong());
            ui.label(egui::RichText::new(subtitle(legend.kind)).color(MUTED));
            ui.add_space(8.0);

            let desired_size = egui::vec2(ui.available_width(), 26.0);
            let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
            let painter = ui.painter();
            let steps = 96;
            for step in 0..steps {
                let t0 = step as f32 / steps as f32;
                let t1 = (step + 1) as f32 / steps as f32;
                let x0 = rect.left() + rect.width() * t0;
                let x1 = rect.left() + rect.width() * t1;
                let band = egui::Rect::from_min_max(
                    egui::pos2(x0, rect.top()),
                    egui::pos2(x1, rect.bottom()),
                );
                painter.rect_filled(band, CornerRadius::ZERO, legend_color(t0 as f64));
            }
            painter.rect_stroke(
                rect,
                CornerRadius::same(4),
                Stroke::new(1.0, BORDER),
                egui::StrokeKind::Outside,
            );

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("{:.3}", legend.min_value))
                        .color(legend_color(0.0)),
                );
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| {
                        ui.label(egui::RichText::new("0.000").color(legend_color(0.5)));
                    },
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{:.3}", legend.max_value))
                            .color(legend_color(1.0)),
                    );
                });
            });

            ui.label(
                egui::RichText::new("Blue is most negative, red is most positive.").color(MUTED),
            );
            ui.label(egui::RichText::new(footer(legend.kind)).color(MUTED));
        });
}

/// Maps a normalized legend position onto the dual tangent color ramp.
fn legend_color(t: f64) -> Color32 {
    let clamped = t.clamp(0.0, 1.0);
    let cold = vector![0.08, 0.22, 1.0];
    let neutral = vector![0.95, 0.95, 1.0];
    let warm = vector![1.0, 0.18, 0.08];
    let color = if clamped < 0.5 {
        let local_mix = clamped * 2.0;
        cold * (1.0 - local_mix) + neutral * local_mix
    } else {
        let local_mix = (clamped - 0.5) * 2.0;
        neutral * (1.0 - local_mix) + warm * local_mix
    };

    Color32::from_rgb(
        (color.x * 255.0).round() as u8,
        (color.y * 255.0).round() as u8,
        (color.z * 255.0).round() as u8,
    )
}

/// Maps one sampled value onto the shared legend ramp and returns RGBA data for rendering.
pub(crate) fn sampled_value_color(value: f64, min_value: f64, max_value: f64) -> Vector4<f64> {
    let mix = if (max_value - min_value).abs() <= 1.0e-6 {
        if value > 1.0e-6 {
            1.0
        } else if value < -1.0e-6 {
            0.0
        } else {
            0.5
        }
    } else {
        ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0)
    };

    let color = legend_color(mix);
    Vector4::new(
        f64::from(color.r()) / 255.0,
        f64::from(color.g()) / 255.0,
        f64::from(color.b()) / 255.0,
        0.95,
    )
}

fn window_title(kind: LegendKind) -> &'static str {
    match kind {
        LegendKind::ScalarField => "Scalar Field Legend",
        LegendKind::DualTangent => "Dual Tangent Legend",
    }
}

fn title(kind: LegendKind) -> &'static str {
    match kind {
        LegendKind::ScalarField => "Scalar Field Legend",
        LegendKind::DualTangent => "Dual Tangent Legend",
    }
}

fn subtitle(kind: LegendKind) -> &'static str {
    match kind {
        LegendKind::ScalarField => "Sampled field values over the current grid",
        LegendKind::DualTangent => "alpha(v) over the sampled dual-space lattice",
    }
}

fn footer(kind: LegendKind) -> &'static str {
    match kind {
        LegendKind::ScalarField => "Visible when rendering the base scalar field.",
        LegendKind::DualTangent => "Visible only in dual tangent mode: Ctrl+T.",
    }
}

/// Returns the viewport id reserved for the detached legend window.
fn legend_viewport_id() -> ViewportId {
    ViewportId::from_hash_of("dual_tangent_legend")
}
