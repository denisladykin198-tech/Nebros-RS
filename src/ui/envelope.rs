// src/ui/envelope.rs
//! UI для огибающей (ADSR) — компактный вид

use egui::Ui;
use crate::utils;
use super::widgets;
use super::theme;
use super::PolivoksApp;

pub fn render(app: &mut PolivoksApp, ui: &mut Ui) {
    ui.vertical(|ui| {
        // ── Общая фиксированная ширина для всего 2-го столбца ──
        let width = ui.available_width().min(super::COL2_CONTENT_WIDTH);
        ui.set_max_width(width);

        widgets::section_header(ui, "📈 ОГИБАЮЩАЯ ADSR");
        widgets::envelope_preview(ui, app.attack, app.decay, app.sustain, app.release);
        ui.add_space(4.0);

        egui::Frame::none()
            .fill(theme::BG_CARD)
            .rounding(6.0)
            .stroke(egui::Stroke::new(1.0, theme::BORDER))
            .inner_margin(8.0)
            .show(ui, |ui| {
                widgets::slider_row(ui, "Атака",   &mut app.attack,  0.001..=5.0, |v| utils::format_time(v));
                widgets::slider_row(ui, "Спад",    &mut app.decay,   0.001..=5.0, |v| utils::format_time(v));
                widgets::slider_row(ui, "Сустейн", &mut app.sustain, 0.0..=1.0,   |v| format!("{:.0}%", v * 100.0));
                widgets::slider_row(ui, "Релиз",   &mut app.release, 0.05..=30.0, |v| utils::format_time(v));
            });
    });
}