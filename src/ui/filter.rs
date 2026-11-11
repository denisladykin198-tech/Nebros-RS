// src/ui/filter.rs
//! Фильтр + артефакты

use egui::Ui;
use super::widgets;
use super::theme;
use super::PolivoksApp;

pub fn render(app: &mut PolivoksApp, ui: &mut Ui) {
    ui.vertical(|ui| {
        // ── Общая фиксированная ширина для всего 2-го столбца ──
        let width = ui.available_width().min(super::COL2_CONTENT_WIDTH);
        ui.set_max_width(width);

        widgets::section_header(ui, "🔻 ФИЛЬТР");
        widgets::filter_curve(ui, app.cutoff, app.resonance);
        ui.add_space(4.0);

        egui::Frame::none()
            .fill(theme::BG_CARD)
            .rounding(6.0)
            .stroke(egui::Stroke::new(1.0, theme::BORDER))
            .inner_margin(8.0)
            .show(ui, |ui| {
                widgets::slider_row(ui, "Частота среза", &mut app.cutoff, 20.0..=8000.0,
                                    |v| format!("{:.0} Гц", v));
                widgets::slider_row(ui, "Резонанс", &mut app.resonance, 0.0..=0.9,
                                    |v| format!("{:.0}%", v * 100.0));
                widgets::slider_row(ui, "Env → Cutoff", &mut app.filter_env_amount, -1.0..=1.0,
                                    |v| format!("{:.0}%", v * 100.0));
                widgets::slider_row(ui, "Клавиатура", &mut app.filter_keyboard_track, 0.0..=1.0,
                                    |v| format!("{:.0}%", v * 100.0));
            });

        ui.add_space(8.0);
        widgets::section_header(ui, "АРТЕФАКТЫ");
        egui::Frame::none()
            .fill(theme::BG_CARD)
            .rounding(6.0)
            .stroke(egui::Stroke::new(1.0, theme::BORDER))
            .inner_margin(8.0)
            .show(ui, |ui| {
                widgets::slider_row(ui, "Температура", &mut app.temperature, 15.0..=60.0,
                                    |v| format!("{:.0}°C", v));
                widgets::slider_row(ui, "Шум питания", &mut app.power_supply_noise, 0.0..=1.0,
                                    |v| format!("{:.0}%", v * 100.0));
                widgets::slider_row(ui, "Старение", &mut app.component_aging, 0.0..=1.0,
                                    |v| format!("{:.0}%", v * 100.0));
            });
    });
}