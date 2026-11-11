//! UI ревербератора с корректным маппингом параметров
use egui::{Color32, Ui};
use parking_lot::Mutex;
use std::sync::Arc;
use crate::synth::PolivoksSynth;

// Макрос с $fmt:literal гарантирует, что строка форматирования будет литералом
macro_rules! reverb_slider {
    ($ui:expr, $label:expr, $params:expr, $idx:expr, $fmt:literal, $mult:expr, $add:expr, $synth:expr) => {{
        $ui.horizontal(|ui| {
            ui.label(format!("{:<18}", $label));
            let mut v = $params[$idx];
            let response = ui.add(egui::Slider::new(&mut v, 0.0..=1.0).text(""));
            
            if response.changed() {
                $params[$idx] = v;
                // parking_lot::Mutex::try_lock возвращает Option, а не Result
                if let Some(mut s) = $synth.try_lock() {
                    s.set_reverb_parameter($idx, v);
                }
            }
            
            let real_val = v * $mult + $add;
            ui.label(format!($fmt, real_val));
        });
    }};
}

/// Универсальный рендер реверба.
pub fn render(
    params: &mut [f32],
    enabled: &mut bool,
    open: &mut bool,
    synth: &Arc<Mutex<PolivoksSynth>>,
    ui: &mut Ui,
) {
    let response = ui.add(egui::widgets::Button::new(if *open {
        "▼ Скрыть настройки реверба"
    } else {
        "▶ Показать настройки реверба"
    }));
    
    if response.clicked() {
        *open = !*open;
    }
    if !*open {
        return;
    }

    ui.group(|ui| {
        ui.horizontal(|ui| {
            if ui.checkbox(enabled, "Включить реверб").changed() {}
            if *enabled {
                ui.colored_label(Color32::GREEN, "✅ АКТИВЕН");
                ui.add_space(10.0);
            } else {
                ui.colored_label(Color32::GRAY, "⭕ ВЫКЛЮЧЕН");
            }
        });
        
        if !*enabled {
            ui.separator();
            ui.label("ℹ️ Включите ревербератор для настройки параметров");
            return;
        }
        ui.separator();

        ui.label("Основные параметры:");
        reverb_slider!(ui, "Смесь (Mix):", params, 0, "{:.0}%", 100.0, 0.0, synth);
        reverb_slider!(ui, "Затухание (Decay):", params, 1, "{:.1} с", 9.9, 0.1, synth);
        reverb_slider!(ui, "Демпфирование:", params, 2, "{:.0}%", 100.0, 0.0, synth);
        
        ui.separator();
        ui.label("Временные параметры:");
        reverb_slider!(ui, "Предзадержка:", params, 3, "{:.0} мс", 200.0, 0.0, synth);
        reverb_slider!(ui, "Размер комнаты:", params, 4, "{:.1}x", 2.5, 0.5, synth);
        reverb_slider!(ui, "Диффузия:", params, 5, "{:.0}%", 100.0, 0.0, synth);
        
        ui.separator();
        ui.label("Тональные параметры:");
        reverb_slider!(ui, "Яркость:", params, 6, "{:.0}%", 100.0, 0.0, synth);
        reverb_slider!(ui, "Модуляция:", params, 7, "{:.0}%", 100.0, 0.0, synth);
        reverb_slider!(ui, "Стерео ширина:", params, 8, "{:.0}%", 100.0, 0.0, synth);
        
        ui.separator();
        ui.label("Артефакты:");
        reverb_slider!(ui, "Шум пружин:", params, 9, "{:.0}%", 100.0, 0.0, synth);
        reverb_slider!(ui, "Натяжение пружин:", params, 10, "{:.0}%", 100.0, 0.0, synth);
        reverb_slider!(ui, "Усталость металла:", params, 11, "{:.0}%", 100.0, 0.0, synth);
        reverb_slider!(ui, "Уровень ржавчины:", params, 12, "{:.0}%", 100.0, 0.0, synth);
        
        ui.separator();
        ui.label("Быстрые пресеты:");
        ui.horizontal(|ui| {
            if ui.button("Небольшая комната").clicked() {
                params.copy_from_slice(&[0.2, 0.15, 0.3, 0.1, 0.3, 0.4, 0.6, 0.0, 0.5, 0.1, 0.5, 0.1, 0.1]);
                if let Some(mut s) = synth.try_lock() {
                    for i in 0..13 { s.set_reverb_parameter(i, params[i]); }
                }
            }
            if ui.button("Большой зал").clicked() {
                params.copy_from_slice(&[0.4, 0.8, 0.2, 0.2, 0.8, 0.7, 0.4, 0.1, 0.8, 0.2, 0.7, 0.4, 0.3]);
                if let Some(mut s) = synth.try_lock() {
                    for i in 0..13 { s.set_reverb_parameter(i, params[i]); }
                }
            }
        });
        ui.horizontal(|ui| {
            if ui.button("Металлический").clicked() {
                params.copy_from_slice(&[0.35, 0.5, 0.1, 0.05, 0.6, 0.3, 0.2, 0.8, 0.3, 0.3, 0.9, 0.9, 0.8]);
                if let Some(mut s) = synth.try_lock() {
                    for i in 0..13 { s.set_reverb_parameter(i, params[i]); }
                }
            }
            if ui.button("Мягкий").clicked() {
                params.copy_from_slice(&[0.25, 0.3, 0.8, 0.15, 0.5, 0.8, 0.7, 0.0, 0.6, 0.0, 0.3, 0.0, 0.0]);
                if let Some(mut s) = synth.try_lock() {
                    for i in 0..13 { s.set_reverb_parameter(i, params[i]); }
                }
            }
        });
    });
}