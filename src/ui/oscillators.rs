// src/ui/oscillators.rs
//! Компактные карточки осцилляторов с превью волны

use egui::{Align, Layout, RichText, Ui};
use crate::synth::WaveShape;
use crate::ui::widgets;
use crate::ui::theme;
use super::PolivoksApp;

pub use super::widgets::slider_row;

pub fn render(app: &mut PolivoksApp, ui: &mut Ui) {
    ui.vertical(|ui| {
        const MAX_PANEL_WIDTH: f32 = 350.0;
        let width = ui.available_width().min(MAX_PANEL_WIDTH);
        ui.set_max_width(width);

        // ── Глобальные настройки (перенесены сюда из filter.rs) ──
        widgets::section_header(ui, "ГЛОБАЛЬНЫЕ");
        egui::Frame::none()
            .fill(theme::BG_CARD)
            .rounding(6.0)
            .stroke(egui::Stroke::new(1.0, theme::BORDER))
            .inner_margin(8.0)
            .show(ui, |ui| {
                slider_row(ui, "Громкость", &mut app.volume, 0.0..=1.0,
                           |v| format!("{:.0}%", v * 100.0));
                slider_row(ui, "Строй A4", &mut app.master_tuning, 430.0..=450.0,
                           |v| format!("{:.1} Гц", v));
                ui.separator();
                if ui.button(RichText::new("Сбросить настройки").color(theme::ACCENT_ORANGE)).clicked() {
                    app.reset_to_defaults();
                }
            });

        ui.add_space(8.0);

        // ── Осцилляторы ──
        widgets::section_header(ui, "🔺 ОСЦИЛЛЯТОРЫ");

        for i in 0..4 {
            ui.push_id(i, |ui| {
                egui::Frame::none()
                    .fill(theme::BG_CARD)
                    .rounding(6.0)
                    .stroke(egui::Stroke::new(1.0, theme::BORDER))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("OSC {}", i + 1))
                                .strong().color(theme::ACCENT_GOLD));
                            widgets::led(ui, app.osc_levels[i] > 0.01,
                                         if app.osc_levels[i] > 0.01 { theme::ACCENT_GREEN } else { theme::TEXT_MUTED });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                egui::ComboBox::from_id_source(format!("wave_{}", i))
                                    .selected_text(match app.osc_wave_shapes[i] {
                                        WaveShape::Sawtooth => "Пила",
                                        WaveShape::Square   => "Меандр",
                                        WaveShape::Triangle => "Треуг.",
                                    })
                                    .width(95.0)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut app.osc_wave_shapes[i], WaveShape::Sawtooth, "Пила");
                                        ui.selectable_value(&mut app.osc_wave_shapes[i], WaveShape::Square,   "Меандр");
                                        ui.selectable_value(&mut app.osc_wave_shapes[i], WaveShape::Triangle, "Треугольник");
                                    });
                            });
                        });

                        widgets::waveform_preview(ui, app.osc_wave_shapes[i], app.osc_levels[i]);

                        slider_row(ui, "Громкость", &mut app.osc_levels[i], 0.0..=1.0,
                                   |v| format!("{:.0}%", v * 100.0));

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Детюн").small().color(theme::TEXT_DIM));
                            ui.spacing_mut().slider_width = 70.0;
                            ui.add(egui::Slider::new(&mut app.osc_detunes[i], -1.0..=1.0).show_value(false));
                            ui.label(RichText::new(format!("{:+.0}c", app.osc_detunes[i] * 100.0))
                                .small().monospace().color(theme::TEXT));
                            ui.add_space(6.0);
                            ui.label(RichText::new("Окт").small().color(theme::TEXT_DIM));
                            ui.spacing_mut().slider_width = 60.0;
                            ui.add(egui::Slider::new(&mut app.osc_octaves[i], -2..=2).show_value(false));
                            ui.label(RichText::new(format!("{:+}", app.osc_octaves[i]))
                                .small().monospace().color(theme::TEXT));
                        });
                    });
                ui.add_space(6.0);
            });
        }
    });
}