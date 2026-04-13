#![allow(unused)]
//! Shared egui colors and styling helpers for the control window.

use eframe::egui::{self, Color32, RichText, Stroke};
use eframe::epaint::{CornerRadius, Margin};

pub(crate) const RASPBERRY: Color32 = Color32::from_rgb(0xB0, 0x18, 0xA2);
pub(crate) const CRAYOLA_BLUE: Color32 = Color32::from_rgb(0x3A, 0x74, 0xE0);
pub(crate) const GOLDEN_GLOW: Color32 = Color32::from_rgb(0xD1, 0xCD, 0x07);
pub(crate) const OLIVE_LEAF: Color32 = Color32::from_rgb(0x61, 0x60, 0x36);
pub(crate) const DARK_KHAKI: Color32 = Color32::from_rgb(0x48, 0x45, 0x2F);
pub(crate) const SHADOW_GREY: Color32 = Color32::from_rgb(0x1E, 0x17, 0x22);
pub(crate) const INK_BLACK: Color32 = Color32::from_rgb(0x08, 0x10, 0x1F);
pub(crate) const JET_BLACK: Color32 = Color32::from_rgb(0x1E, 0x26, 0x33);

pub(crate) const PANEL: Color32 = INK_BLACK;
pub(crate) const BORDER: Color32 = DARK_KHAKI;
pub(crate) const ACCENT: Color32 = CRAYOLA_BLUE;
pub(crate) const TEXT: Color32 = Color32::from_rgb(0xE9, 0xEF, 0xFE);
pub(crate) const MUTED: Color32 = Color32::from_rgb(175, 182, 195);

/// Applies the shared egui theme used by the control window.
pub(crate) fn apply_style(ctx: &egui::Context) {
    ctx.set_visuals(egui::Visuals::dark());
    let mut style = (*ctx.global_style()).clone();
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
    ctx.set_global_style(style);
}

/// Builds a styled section heading for collapsible UI groups.
pub(crate) fn section_heading(text: &str) -> RichText {
    RichText::new(text).strong()
}
