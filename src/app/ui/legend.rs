//! Detached legend window and shared color ramp for sampled scalar or dual-form values.

use crate::app::ui::state::LegendState;
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
    ctx.send_viewport_cmd_to(
        legend_viewport_id(),
        ViewportCommand::Title(legend.kind.descriptor().window_title.to_string()),
    );

    let mut builder = ViewportBuilder::default()
        .with_title(legend.kind.descriptor().window_title)
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
            let descriptor = legend.kind.descriptor();
            ui.label(egui::RichText::new(descriptor.title).color(TEXT).strong());
            ui.label(egui::RichText::new(descriptor.subtitle).color(MUTED));
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
            render_scale_labels(ui, legend);

            ui.label(
                egui::RichText::new("Blue is most negative, red is most positive.").color(MUTED),
            );
            ui.label(egui::RichText::new(descriptor.footer).color(MUTED));
        });
}

/// Renders scale markers underneath the legend ramp using the actual value range.
fn render_scale_labels(ui: &mut egui::Ui, legend: LegendState) {
    let desired_size = egui::vec2(ui.available_width(), 20.0);
    let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
    let painter = ui.painter();

    for marker in legend_markers(legend) {
        let x = rect.left() + rect.width() * marker.mix as f32;
        painter.text(
            egui::pos2(x, rect.center().y),
            marker.anchor,
            marker.label,
            egui::FontId::proportional(12.0),
            marker.color,
        );
    }
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

/// Maps a scalar value onto the normalized legend range.
fn legend_mix(value: f64, min_value: f64, max_value: f64) -> f64 {
    if (max_value - min_value).abs() <= 1.0e-6 {
        if value > 1.0e-6 {
            1.0
        } else if value < -1.0e-6 {
            0.0
        } else {
            0.5
        }
    } else {
        ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0)
    }
}

/// Maps one sampled value onto the shared legend ramp and returns RGBA data for rendering.
pub(crate) fn sampled_value_color(value: f64, min_value: f64, max_value: f64) -> Vector4<f64> {
    let mix = legend_mix(value, min_value, max_value);

    let color = legend_color(mix);
    Vector4::new(
        f64::from(color.r()) / 255.0,
        f64::from(color.g()) / 255.0,
        f64::from(color.b()) / 255.0,
        0.95,
    )
}

struct LegendMarker {
    mix: f64,
    label: String,
    color: Color32,
    anchor: egui::Align2,
}

fn legend_markers(legend: LegendState) -> Vec<LegendMarker> {
    let mut markers = vec![
        LegendMarker {
            mix: 0.0,
            label: format!("{:.3}", legend.min_value),
            color: legend_color(0.0),
            anchor: egui::Align2::LEFT_CENTER,
        },
        LegendMarker {
            mix: 1.0,
            label: format!("{:.3}", legend.max_value),
            color: legend_color(1.0),
            anchor: egui::Align2::RIGHT_CENTER,
        },
    ];

    let middle_value = if legend.min_value < 0.0 && legend.max_value > 0.0 {
        0.0
    } else {
        0.5 * (legend.min_value + legend.max_value)
    };
    let middle_mix = legend_mix(middle_value, legend.min_value, legend.max_value);

    markers.push(LegendMarker {
        mix: middle_mix,
        label: format!("{:.3}", middle_value),
        color: legend_color(middle_mix),
        anchor: egui::Align2::CENTER_CENTER,
    });

    markers
}

#[cfg(test)]
mod tests {
    use super::{legend_markers, legend_mix};
    use crate::app::ui::{LegendKind, LegendState};

    #[test]
    fn legend_mix_places_zero_at_start_for_nonnegative_range() {
        assert_eq!(legend_mix(0.0, 0.0, 10.0), 0.0);
    }

    #[test]
    fn legend_markers_use_midpoint_when_zero_is_not_inside_range() {
        let markers = legend_markers(LegendState {
            kind: LegendKind::ScalarField,
            min_value: 0.0,
            max_value: 10.0,
        });

        assert_eq!(markers[0].label, "0.000");
        assert_eq!(markers[1].label, "10.000");
        assert_eq!(markers[2].label, "5.000");
        assert_eq!(markers[2].mix, 0.5);
    }
}

/// Returns the viewport id reserved for the detached legend window.
fn legend_viewport_id() -> ViewportId {
    ViewportId::from_hash_of("dual_tangent_legend")
}
