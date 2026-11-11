// src/ui/voice_modes.rs
//! Компактный режим голосоведения и статус

use egui::{RichText, Ui};
use crate::synth::VoiceMode;
use crate::utils;
use super::widgets;
use super::theme;
use super::PolivoksApp;

pub fn render(app: &mut PolivoksApp, ui: &mut Ui) {
    ui.vertical(|ui| {
        // ── Общая фиксированная ширина для всего 2-го столбца ──
        let width = ui.available_width().min(super::COL2_CONTENT_WIDTH);
        ui.set_max_width(width);

        widgets::section_header(ui, "ГОЛОСОВЕДЕНИЕ");

        egui::Frame::none()
            .fill(theme::BG_CARD)
            .rounding(6.0)
            .stroke(egui::Stroke::new(1.0, theme::BORDER))
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Режим").small().color(theme::TEXT_DIM));
                    egui::ComboBox::from_id_source("voice_mode")
                        .selected_text(match app.voice_mode {
                            VoiceMode::Monophonic => "Моно",
                            VoiceMode::Legato     => "Легато",
                            VoiceMode::Polyphonic => "Поли",
                            VoiceMode::Portamento => "Портаменто",
                        })
                        .width(130.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut app.voice_mode, VoiceMode::Monophonic, "Монофония");
                            ui.selectable_value(&mut app.voice_mode, VoiceMode::Legato,     "Легато");
                            ui.selectable_value(&mut app.voice_mode, VoiceMode::Polyphonic, "Полифония (8)");
                            ui.selectable_value(&mut app.voice_mode, VoiceMode::Portamento, "Портаменто");
                        });
                });

                if app.voice_mode == VoiceMode::Portamento {
                    widgets::slider_row(ui, "Портаменто", &mut app.portamento_speed, 0.01..=1.0,
                                        |v| utils::format_time(v));
                }

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Retrigger").small().color(theme::TEXT_DIM));
                    ui.checkbox(&mut app.retrigger_envelope, "");
                    if app.retrigger_envelope {
                        ui.colored_label(theme::ACCENT_GREEN, RichText::new("ВКЛ").small().strong());
                    } else {
                        ui.label(RichText::new("ВЫКЛ").small().color(theme::TEXT_MUTED));
                    }
                });
            });

        ui.add_space(8.0);
        widgets::section_header(ui, "СТАТУС");

        egui::Frame::none()
            .fill(theme::BG_CARD)
            .rounding(6.0)
            .stroke(egui::Stroke::new(1.0, theme::BORDER))
            .inner_margin(8.0)
            .show(ui, |ui| {
                let synth = app.synth.lock();
                let active_notes = synth.active_notes();

                ui.horizontal(|ui| {
                    widgets::led(ui, !active_notes.is_empty(),
                                 if !active_notes.is_empty() { theme::ACCENT_GREEN } else { theme::TEXT_MUTED });
                    ui.label(RichText::new(format!("{:?}", synth.voice_mode()))
                        .small().color(theme::TEXT_DIM));
                });

                if active_notes.is_empty() {
                    ui.label(RichText::new("тишина · нажмите клавишу")
                        .small().italics().color(theme::TEXT_MUTED));
                } else {
                    if let Some(freq) = active_notes.first() {
                        let name = utils::freq_to_note_name(*freq);
                        ui.label(RichText::new(format!("{} · {:.1} Hz", name, freq))
                            .strong().color(theme::ACCENT_GOLD));
                    }
                    if active_notes.len() >= 2 {
                        ui.label(RichText::new(format!("Аккорд: {}", utils::get_chord_name(&active_notes)))
                            .small().color(theme::ACCENT_GREEN));
                    }
                    ui.label(RichText::new(format!("Env: {:.2}", synth.current_envelope()))
                        .small().color(theme::TEXT_DIM));
                }
            });
    });
}