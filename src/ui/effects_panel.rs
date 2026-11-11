// src/ui/effects_panel.rs
use egui::{RichText, Ui};
use super::ui_effect::EffectType;
use super::reverb;
use super::widgets;
use super::theme;
use super::PolivoksApp;

pub fn render(app: &mut PolivoksApp, ui: &mut Ui) {
    ui.vertical(|ui| {

        const MAX_PANEL_WIDTH: f32 = 350.0;
        let width = ui.available_width().min(MAX_PANEL_WIDTH);
        ui.set_max_width(width);


        widgets::section_header(ui, "ЭФФЕКТЫ");

        ui.horizontal(|ui| {
            if ui.button(RichText::new("📂 Открыть меню").small()).clicked() {
                app.effects_menu_open = !app.effects_menu_open;
            }
            let active_count = app.effects.iter().filter(|e| e.enabled).count();
            ui.label(RichText::new(format!("активных: {}", active_count))
                .small().color(if active_count > 0 { theme::ACCENT_GREEN } else { theme::TEXT_MUTED }));
        });

        if app.effects_menu_open {
            show_effects_menu(app, ui);
        }

        ui.add_space(6.0);

        // Список активных эффектов
        let active: Vec<(usize, String)> = app.effects.iter().enumerate()
            .filter(|(_, e)| e.enabled)
            .map(|(i, e)| (i, e.name.clone()))
            .collect();

        if !active.is_empty() {
            egui::Frame::none()
                .fill(theme::BG_CARD)
                .rounding(6.0)
                .stroke(egui::Stroke::new(1.0, theme::BORDER))
                .inner_margin(6.0)
                .show(ui, |ui| {
                    for (i, name) in &active {
                        ui.horizontal(|ui| {
                            widgets::led(ui, true, theme::ACCENT_GREEN);
                            if ui.selectable_label(app.selected_effect == Some(*i),
                                                   RichText::new(name).small().color(theme::TEXT)).clicked() {
                                app.selected_effect = Some(*i);
                                app.effects[*i].open = true;
                            }
                        });
                    }
                    ui.separator();
                    if ui.button(RichText::new("Выключить все").small()).clicked() {
                        for e in &mut app.effects { e.enabled = false; }
                        app.selected_effect = None;
                    }
                });
        }

        show_selected_effect_panel(app, ui);
    });
}

fn show_effects_menu(app: &mut PolivoksApp, ui: &mut Ui) {
    let mut open = app.effects_menu_open;
    let mut selected_change: Option<usize> = None;

        const MAX_PANEL_WIDTH: f32 = 350.0;
        let width = ui.available_width().min(MAX_PANEL_WIDTH);
        ui.set_max_width(width);


    egui::Window::new(RichText::new("Меню эффектов").color(theme::ACCENT_GOLD))
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(320.0)
        .show(ui.ctx(), |ui| {
            ui.label(RichText::new("Отметьте активные · клик по имени — настройка")
                .small().color(theme::TEXT_DIM));
            ui.separator();
            egui::ScrollArea::vertical().max_height(340.0).show(ui, |ui| {
                for i in 0..app.effects.len() {
                    let name = app.effects[i].name.clone();
                    let mut enabled = app.effects[i].enabled;
                    let is_selected = app.selected_effect == Some(i);
                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut enabled, "").changed() {
                            app.effects[i].enabled = enabled;
                        }
                        if ui.selectable_label(is_selected, RichText::new(&name).small()).clicked() {
                            selected_change = Some(i);
                        }
                    });
                }
            });
        });

    if let Some(i) = selected_change {
        app.selected_effect = Some(i);
        app.effects[i].open = true;
    }
    app.effects_menu_open = open;
}

fn show_selected_effect_panel(app: &mut PolivoksApp, ui: &mut Ui) {
    let Some(idx) = app.selected_effect else { return; };
    if idx >= app.effects.len() { return; }

    let effect_type = app.effects[idx].effect_type.clone();
    let name = app.effects[idx].name.clone();
    let enabled = app.effects[idx].enabled;

    ui.add_space(6.0);
    let mut open = app.effects[idx].open;
    let header_text = if open { format!("▼ {}", name) } else { format!("▶ {}", name) };
    if ui.button(RichText::new(header_text).small()).clicked() {
        open = !open;
        app.effects[idx].open = open;
    }
    if !open { return; }

    egui::Frame::none()
        .fill(theme::BG_CARD)
        .rounding(6.0)
        .stroke(egui::Stroke::new(1.0, theme::BORDER))
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                widgets::led(ui, enabled,
                             if enabled { theme::ACCENT_GREEN } else { theme::TEXT_MUTED });
                ui.label(RichText::new(if enabled { "ВКЛ" } else { "ВЫКЛ" })
                    .small().color(if enabled { theme::ACCENT_GREEN } else { theme::TEXT_MUTED }));
                let mut en = enabled;
                if ui.checkbox(&mut en, "").changed() {
                    app.effects[idx].enabled = en;
                }
            });
            ui.separator();

            if !enabled {
                ui.label(RichText::new("включите для настройки")
                    .small().italics().color(theme::TEXT_MUTED));
                return;
            }

            match effect_type {
                EffectType::Reverb => {
                    let effect = &mut app.effects[idx];
                    reverb::render(&mut effect.parameters, &mut effect.enabled,
                                   &mut effect.open, &app.synth, ui);
                }
                EffectType::Distortion | EffectType::Chorus
                | EffectType::Delay | EffectType::Generic => {
                    let params = &mut app.effects[idx].parameters;
                    for (pi, p) in params.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("П{}", pi + 1))
                                .small().color(theme::TEXT_DIM));
                            ui.spacing_mut().slider_width = 80.0;
                            ui.add(egui::Slider::new(p, 0.0..=1.0).show_value(false));
                            ui.label(RichText::new(format!("{:.0}%", *p * 100.0))
                                .small().monospace().color(theme::TEXT));
                        });
                    }
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Случайные").small()).clicked() {
                            use std::time::{SystemTime, UNIX_EPOCH};
                            let seed = SystemTime::now().duration_since(UNIX_EPOCH)
                                .unwrap_or_default().as_nanos() as u64;
                            for (i, p) in params.iter_mut().enumerate() {
                                let x = seed.wrapping_mul(6364136223846793005)
                                    .wrapping_add(i as u64 * 1442695040888963407);
                                *p = ((x >> 32) % 1001) as f32 / 1000.0;
                            }
                        }
                    });
                }
            }
        });
}