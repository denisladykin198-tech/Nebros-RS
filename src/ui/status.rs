// src/ui/status.rs
//! Статус аудио — LED-стиль

use egui::{RichText, Ui};
use super::theme;
use super::widgets;
use super::PolivoksApp;

pub fn render_audio_status(app: &mut PolivoksApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        widgets::led(ui, app.audio_stream.is_some(),
                     if app.audio_stream.is_some() { theme::ACCENT_GREEN } else { theme::ACCENT_RED });
        let color = if app.audio_status.contains("✅") { theme::ACCENT_GREEN }
                    else if app.audio_status.contains("⚠") { theme::ACCENT_ORANGE }
                    else { theme::TEXT_DIM };
        ui.label(RichText::new(&app.audio_status).small().color(color));
        if ui.button(RichText::new("🔄").small()).clicked() {
            app.manual_restart_requested = true;
        }
    });
}