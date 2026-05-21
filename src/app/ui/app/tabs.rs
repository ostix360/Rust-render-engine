use super::{ControlApp, PresetLabel};
use crate::app::ui::presets::{EmPreset, FieldPreset, GridPreset};
use crate::app::ui::state::{ControlTab, EmGauge, EmMode, FieldKind, GridUiState};
use crate::app::ui::theme::{self, MUTED, RASPBERRY, TEXT};
use eframe::egui::{self, Color32};
use eframe::epaint::CornerRadius;

impl ControlApp {
    /// Dispatches to the active tab renderer.
    pub(super) fn render_active_tab(
        ui: &mut egui::Ui,
        active_tab: ControlTab,
        data: &mut GridUiState,
    ) {
        match active_tab {
            ControlTab::Grid => Self::render_grid_tab(ui, data),
            ControlTab::Field => Self::render_field_tab(ui, data),
            ControlTab::Em => Self::render_em_tab(ui, data),
        }
    }

    /// Renders the grid, coordinate-system, and tangent-scale controls.
    fn render_grid_tab(ui: &mut egui::Ui, data: &mut GridUiState) {
        egui::CollapsingHeader::new(theme::section_heading("Standard parameters"))
            .default_open(true)
            .show(ui, |ui| {
                Self::preset_buttons(ui, GridPreset::ALL, |preset, data| preset.apply(data), data);
            });

        ui.add_space(8.0);
        egui::CollapsingHeader::new(theme::section_heading("Coordinate system"))
            .default_open(true)
            .show(ui, |ui| {
                ui.checkbox(
                    &mut data.render_3d,
                    egui::RichText::new("Render 3D").color(TEXT),
                );
                ui.separator();
                Self::eq_row(ui, "Equation x:  x =", &mut data.coords_sys.x.eq_str);
                Self::eq_row(ui, "Equation y:  y =", &mut data.coords_sys.y.eq_str);
                Self::eq_row(ui, "Equation z:  z =", &mut data.coords_sys.z.eq_str);
            });

        ui.add_space(8.0);
        egui::CollapsingHeader::new(theme::section_heading("Grid settings"))
            .default_open(true)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Line per coordinate").color(MUTED));
                Self::density_slider(ui, &mut data.nb_x, "x");
                Self::density_slider(ui, &mut data.nb_y, "y");
                Self::density_slider(ui, &mut data.nb_z, "z");

                ui.separator();
                ui.label(egui::RichText::new("Bounds").color(MUTED));
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

        ui.add_space(8.0);
        egui::CollapsingHeader::new(theme::section_heading("Tangent view"))
            .default_open(true)
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("Local patch size in geometric tangent mode").color(MUTED),
                );
                ui.add(
                    egui::Slider::new(&mut data.tangent_scale, 0.02..=0.5)
                        .logarithmic(true)
                        .text("local scale")
                        .trailing_fill(true),
                );
                ui.label(
                    egui::RichText::new(
                        "This updates live and only affects tangent-space rendering.",
                    )
                    .color(MUTED),
                );
            });
    }

    /// Renders the field-equation and tangent-arrow controls.
    fn render_field_tab(ui: &mut egui::Ui, data: &mut GridUiState) {
        egui::CollapsingHeader::new(theme::section_heading("Standard parameters"))
            .default_open(true)
            .show(ui, |ui| {
                Self::preset_buttons(
                    ui,
                    FieldPreset::ALL,
                    |preset, data| preset.apply(data),
                    data,
                );
            });

        ui.add_space(8.0);
        egui::CollapsingHeader::new(theme::section_heading("Field equations"))
            .default_open(true)
            .show(ui, |ui| {
                Self::render_field_kind_selector(ui, &mut data.field_kind);
                ui.add_space(8.0);
                ui.checkbox(&mut data.render_d, egui::RichText::new("Apply d").color(TEXT));
                ui.separator();

                match data.field_kind {
                    FieldKind::Scalar => {
                        ui.label(
                            egui::RichText::new("Scalar field in the active coordinates")
                                .color(MUTED),
                        );
                        Self::eq_row(ui, "Equation:  f =", &mut data.scalar_field.eq_str);
                        ui.label(
                            egui::RichText::new(
                                "Base render uses colored samples. Enabling d renders the gradient.",
                            )
                            .color(MUTED),
                        );
                    }
                    FieldKind::Vector => {
                        ui.label(
                            egui::RichText::new("Vector components in the active coordinates")
                                .color(MUTED),
                        );
                        Self::eq_row(ui, "Equation x:  Fx =", &mut data.field.x.eq_str);
                        Self::eq_row(ui, "Equation y:  Fy =", &mut data.field.y.eq_str);
                        Self::eq_row(ui, "Equation z:  Fz =", &mut data.field.z.eq_str);
                        ui.label(
                            egui::RichText::new(
                                "Base render uses arrows. Enabling d renders the associated curl field.",
                            )
                            .color(MUTED),
                        );
                    }
                }

                ui.add_space(8.0);
                ui.add_enabled_ui(data.renders_vector_field(), |ui| {
                    ui.checkbox(
                        &mut data.normalize_field,
                        egui::RichText::new("Normalize field").color(TEXT),
                    );
                });
                if data.renders_scalar_samples() {
                    ui.label(
                        egui::RichText::new(
                            "Normalization is available only when the current render uses arrows.",
                        )
                        .color(MUTED),
                    );
                }
            });

        ui.add_space(8.0);
        egui::CollapsingHeader::new(theme::section_heading("Tangent arrows"))
            .default_open(true)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Arrow size in geometric tangent mode").color(MUTED));
                ui.add(
                    egui::Slider::new(&mut data.geometric_arrow_scale, 0.1..=1.5)
                        .logarithmic(true)
                        .text("arrow scale")
                        .trailing_fill(true),
                );
                ui.label(
                    egui::RichText::new(
                        "This updates live and only affects geometric tangent-space arrows.",
                    )
                    .color(MUTED),
                );
            });
    }

    fn render_em_tab(ui: &mut egui::Ui, data: &mut GridUiState) {
        egui::CollapsingHeader::new(theme::section_heading("Standard parameters"))
            .default_open(true)
            .show(ui, |ui| {
                Self::preset_buttons(ui, EmPreset::ALL, |preset, data| preset.apply(data), data);
            });

        ui.add_space(8.0);
        egui::CollapsingHeader::new(theme::section_heading("Electromagnetism"))
            .default_open(true)
            .show(ui, |ui| {
                ui.checkbox(
                    &mut data.em.enabled,
                    egui::RichText::new("Enable EM").color(TEXT),
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Mode").color(TEXT));
                    if Self::tab_button(ui, data.em.mode == EmMode::Potentials, "Potentials") {
                        data.em.mode = EmMode::Potentials;
                    }
                    if Self::tab_button(ui, data.em.mode == EmMode::Electric, "E") {
                        data.em.mode = EmMode::Electric;
                    }
                    if Self::tab_button(ui, data.em.mode == EmMode::Magnetic, "B") {
                        data.em.mode = EmMode::Magnetic;
                    }
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Gauge").color(TEXT));
                    if Self::tab_button(ui, data.em.gauge == EmGauge::Coulomb, "Coulomb") {
                        data.em.gauge = EmGauge::Coulomb;
                    }
                    // if Self::tab_button(ui, data.em.gauge == EmGauge::Lorenz, "Lorenz", ) {
                    //     data.em.gauge = EmGauge::Lorenz;
                    // }
                });
                ui.label(
                    egui::RichText::new(
                        "Named gauge selector for potential display/reconstruction. Coulomb-like \
                         is the current reconstruction",
                    )
                    .color(MUTED),
                );
                ui.separator();
                Self::em_source_group(ui, data.em.mode == EmMode::Potentials, |ui| {
                    Self::eq_row(ui, "Scalar:  V =", &mut data.em.phi.eq_str);
                    Self::eq_row(
                        ui,
                        "Vector x:  Ax =",
                        &mut data.em.vector_potential.x.eq_str,
                    );
                    Self::eq_row(
                        ui,
                        "Vector y:  Ay =",
                        &mut data.em.vector_potential.y.eq_str,
                    );
                    Self::eq_row(
                        ui,
                        "Vector z:  Az =",
                        &mut data.em.vector_potential.z.eq_str,
                    );
                });
                ui.separator();
                Self::em_source_group(ui, data.em.mode == EmMode::Electric, |ui| {
                    Self::eq_row(
                        ui,
                        "Electric x:  Ex =",
                        &mut data.em.electric_field.x.eq_str,
                    );
                    Self::eq_row(
                        ui,
                        "Electric y:  Ey =",
                        &mut data.em.electric_field.y.eq_str,
                    );
                    Self::eq_row(
                        ui,
                        "Electric z:  Ez =",
                        &mut data.em.electric_field.z.eq_str,
                    );
                });
                ui.separator();
                Self::em_source_group(ui, data.em.mode == EmMode::Magnetic, |ui| {
                    Self::eq_row(
                        ui,
                        "Magnetic x:  Bx =",
                        &mut data.em.magnetic_field.x.eq_str,
                    );
                    Self::eq_row(
                        ui,
                        "Magnetic y:  By =",
                        &mut data.em.magnetic_field.y.eq_str,
                    );
                    Self::eq_row(
                        ui,
                        "Magnetic z:  Bz =",
                        &mut data.em.magnetic_field.z.eq_str,
                    );
                });
            });

        ui.add_space(8.0);
        egui::CollapsingHeader::new(theme::section_heading("Constants"))
            .default_open(true)
            .show(ui, |ui| {
                ui.add(
                    egui::Slider::new(&mut data.em.light_speed, 0.1..=100.0)
                        .logarithmic(true)
                        .text("c")
                        .trailing_fill(true),
                );
                ui.add(
                    egui::Slider::new(&mut data.em.magnetic_vector_scale, 0.1..=100.0)
                        .logarithmic(true)
                        .text("B vector scale")
                        .trailing_fill(true),
                );
                ui.checkbox(
                    &mut data.em.normalize_vectors,
                    egui::RichText::new("Normalize EM vectors").color(TEXT),
                );
            });

        ui.add_space(8.0);
        egui::CollapsingHeader::new(theme::section_heading("Time"))
            .default_open(true)
            .show(ui, |ui| {
                ui.checkbox(&mut data.em.running, egui::RichText::new("Run").color(TEXT));
                ui.add(
                    egui::Slider::new(&mut data.em.time_scale, -5.0..=5.0)
                        .text("time scale")
                        .trailing_fill(true),
                );
                if ui
                    .add(
                        egui::Button::new(egui::RichText::new("Reset time").color(Color32::WHITE))
                            .fill(RASPBERRY)
                            .min_size(egui::vec2(120.0, 30.0))
                            .corner_radius(CornerRadius::same(6)),
                    )
                    .clicked()
                {
                    data.em.reset_counter += 1;
                }
            });

        ui.add_space(8.0);
        egui::CollapsingHeader::new(theme::section_heading("Layers"))
            .default_open(true)
            .show(ui, |ui| {
                ui.checkbox(
                    &mut data.em.layers.electric,
                    egui::RichText::new("E").color(TEXT),
                );
                ui.checkbox(
                    &mut data.em.layers.magnetic,
                    egui::RichText::new("B").color(TEXT),
                );
                ui.checkbox(
                    &mut data.em.layers.scalar_potential,
                    egui::RichText::new("V").color(TEXT),
                )
                .on_hover_text(
                    "Shows the scalar potential. The built-in wave presets use the V = 0 gauge, \
                     so this layer is intentionally uniform until V is edited or reconstructed \
                     from E/B source mode.",
                );
                ui.checkbox(
                    &mut data.em.layers.vector_potential,
                    egui::RichText::new("A").color(TEXT),
                );
            });
    }

    fn em_source_group(
        ui: &mut egui::Ui,
        editable: bool,
        add_contents: impl FnOnce(&mut egui::Ui),
    ) {
        ui.add_enabled_ui(editable, add_contents);
    }

    fn preset_buttons<T: Copy>(
        ui: &mut egui::Ui,
        presets: impl IntoIterator<Item = T>,
        apply: impl Fn(T, &mut GridUiState),
        data: &mut GridUiState,
    ) where
        T: PresetLabel,
    {
        ui.horizontal_wrapped(|ui| {
            for preset in presets {
                if Self::compact_button(ui, preset.label()) {
                    apply(preset, data);
                }
            }
        });
    }
}
