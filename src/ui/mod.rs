// src/ui/mod.rs
//! Главное окно приложения

pub mod theme;
pub mod widgets;
pub mod ui_effect;
pub mod oscillators;
pub mod filter;
pub mod envelope;
pub mod voice_modes;
pub mod reverb;
pub mod effects_panel;
pub mod status;
pub use theme::*;

use egui::{Color32, Layout, RichText, Vec2};
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::audio;
use crate::consts::FRAME_DURATION_MS;
use crate::presets;
use crate::synth::{PolivoksSynth, WaveShape, VoiceMode};
use crate::ui::ui_effect::EffectType;
use crate::utils;

use ui_effect::UiEffect;

pub(crate) const COL2_CONTENT_WIDTH: f32 = 350.0;

pub struct PolivoksApp {
    pub(crate) synth: Arc<Mutex<PolivoksSynth>>,
    pub(crate) active_keys: Vec<(egui::Key, f32)>,

    pub(crate) osc_wave_shapes: [WaveShape; 4],
    pub(crate) osc_levels: [f32; 4],
    pub(crate) osc_detunes: [f32; 4],
    pub(crate) osc_octaves: [i32; 4],

    pub(crate) attack: f32,
    pub(crate) decay: f32,
    pub(crate) sustain: f32,
    pub(crate) release: f32,

    pub(crate) cutoff: f32,
    pub(crate) resonance: f32,
    pub(crate) filter_env_amount: f32,
    pub(crate) filter_keyboard_track: f32,

    pub(crate) temperature: f32,
    pub(crate) power_supply_noise: f32,
    pub(crate) component_aging: f32,

    pub(crate) voice_mode: VoiceMode,
    pub(crate) portamento_speed: f32,
    pub(crate) retrigger_envelope: bool,

    pub(crate) volume: f32,
    pub(crate) master_tuning: f32,

    pub(crate) effects: Vec<UiEffect>,
    pub(crate) effects_menu_open: bool,
    pub(crate) selected_effect: Option<usize>,

    /// Окно быстрых пресетов (открывается из верхней панели)
    pub(crate) presets_menu_open: bool,

    pub(crate) audio_stream: Option<cpal::Stream>,
    pub(crate) audio_error_flag: Arc<AtomicBool>,
    audio_restart_timer: f64,
    pub(crate) audio_status: String,
    manual_restart_requested: bool,
    current_device_name: Option<String>,
    device_check_timer: f64,
    last_device_change_time: f64,
    /// MIDI-нота, зажатая мышью на экранной клавиатуре (для парного note_off)
    pub(crate) mouse_pressed_note: Option<u8>,
    /// Октава, которой управляет компьютерная клавиатура (0 = C2, 1 = C3, 2 = C4, 3 = C5)
    pub(crate) keyboard_octave: u8,
}


impl PolivoksApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        log::info!("Инициализация UI...");
        theme::apply(&cc.egui_ctx);

        let synth = Arc::new(Mutex::new(PolivoksSynth::new()));
        let audio_error_flag = Arc::new(AtomicBool::new(false));

        let (audio_stream, status, device_name) = audio::start_audio(
            Arc::clone(&synth),
            Arc::clone(&audio_error_flag),
        );

        let mut effects = presets::default_effects();
        if !effects.iter().any(|e| e.effect_type == EffectType::Reverb) {
            effects.insert(0, UiEffect::new("Пружинный реверб", EffectType::Reverb));
        }
        if let Some(first) = effects.first_mut() { first.enabled = true; first.open = true; }

        Self {
            synth,
            active_keys: Vec::new(),
            osc_wave_shapes: [WaveShape::Sawtooth, WaveShape::Square, WaveShape::Triangle, WaveShape::Sawtooth],
            osc_levels: [1.0, 0.5, 0.3, 0.0],
            osc_detunes: [0.0; 4],
            osc_octaves: [0; 4],
            attack: 0.01, decay: 0.1, sustain: 0.7, release: 0.5,
            cutoff: 1000.0, resonance: 0.3,
            filter_env_amount: 0.5, filter_keyboard_track: 0.5,
            temperature: 25.0, power_supply_noise: 0.1, component_aging: 0.0,
            voice_mode: VoiceMode::Monophonic,
            portamento_speed: 0.1, retrigger_envelope: false,
            volume: 0.7, master_tuning: 440.0,
            effects, effects_menu_open: false, selected_effect: Some(0),
            presets_menu_open: false,
            audio_stream, audio_error_flag,
            audio_restart_timer: 0.0, audio_status: status,
            manual_restart_requested: false,
            current_device_name: device_name,
            device_check_timer: 0.0,
            last_device_change_time: 0.0,
            mouse_pressed_note: None,
            keyboard_octave: 2,
        }
    }

    pub(crate) fn update_synth_params(&self) {
        let mut synth = self.synth.lock();
        for i in 0..4 {
            synth.set_osc_wave_shape(i, self.osc_wave_shapes[i]);
            synth.set_osc_level(i, self.osc_levels[i]);
            synth.set_osc_detune(i, self.osc_detunes[i]);
            synth.set_osc_octave(i, self.osc_octaves[i]);
        }
        synth.set_adsr(self.attack, self.decay, self.sustain, self.release);
        synth.set_cutoff(self.cutoff);
        synth.set_resonance(self.resonance);
        synth.set_filter_env_amount(self.filter_env_amount);
        synth.set_filter_keyboard_track(self.filter_keyboard_track);
        synth.set_temperature(self.temperature);
        synth.set_power_noise(self.power_supply_noise);
        synth.set_component_aging(self.component_aging);
        synth.set_master_tuning(self.master_tuning);
        synth.set_voice_mode(self.voice_mode);
        if self.voice_mode == VoiceMode::Portamento {
            synth.set_portamento_speed(self.portamento_speed);
        }
        synth.set_retrigger_envelope(self.retrigger_envelope);
        synth.set_reverb_enabled(false);
        for effect in &self.effects {
            if effect.effect_type == EffectType::Reverb && effect.enabled {
                synth.set_reverb_enabled(true);
                for (i, &p) in effect.parameters.iter().enumerate() {
                    synth.set_reverb_parameter(i, p);
                }
            }
        }
        synth.set_volume(self.volume);
    }

    fn check_audio_status(&mut self, frame_dt: f64) {
        if self.audio_error_flag.load(Ordering::SeqCst) {
            self.audio_status = "⚠️ Аудио ошибка, восстановление...".to_string();
            self.audio_error_flag.store(false, Ordering::SeqCst);
            if self.audio_restart_timer <= 0.0 { self.do_restart_audio(); }
        }
        if self.last_device_change_time > 0.0 {
            self.last_device_change_time -= frame_dt;
        } else {
            self.device_check_timer -= frame_dt;
            if self.device_check_timer <= 0.0 {
                self.device_check_timer = 1.0;
                if let Some(new_name) = audio::device::check_current_device(&self.current_device_name) {
                    self.audio_status = format!("🔄 Переключение: {}", new_name);
                    self.last_device_change_time = 3.0;
                    self.manual_restart_requested = true;
                }
            }
        }
        if self.audio_stream.is_none() && self.audio_restart_timer <= 0.0 {
            self.do_restart_audio();
        }
        if self.audio_restart_timer > 0.0 { self.audio_restart_timer -= frame_dt; }
        if self.manual_restart_requested && self.audio_restart_timer <= 0.0 {
            self.do_restart_audio();
        }
    }

    fn do_restart_audio(&mut self) {
        let (stream, status, new_device) = audio::restart_audio(
            &mut self.audio_stream, Arc::clone(&self.synth),
            Arc::clone(&self.audio_error_flag), &self.current_device_name,
        );
        self.audio_stream = stream;
        self.audio_status = status;
        self.current_device_name = new_device;
        self.audio_restart_timer = 2.0;
        self.device_check_timer = 0.0;
        self.last_device_change_time = 5.0;
        self.manual_restart_requested = false;
    }

    pub(crate) fn apply_preset(&mut self, preset: presets::PresetData) {
        self.osc_wave_shapes = preset.osc_wave_shapes;
        self.osc_levels = preset.osc_levels;
        self.osc_detunes = preset.osc_detunes;
        self.osc_octaves = preset.osc_octaves;
        self.attack = preset.attack;
        self.decay = preset.decay;
        self.sustain = preset.sustain;
        self.release = preset.release;
        self.cutoff = preset.cutoff;
        self.resonance = preset.resonance;
        self.voice_mode = preset.voice_mode;
        self.retrigger_envelope = preset.retrigger_envelope;
    }

    pub(crate) fn reset_to_defaults(&mut self) {
        self.osc_wave_shapes = [WaveShape::Sawtooth, WaveShape::Square, WaveShape::Triangle, WaveShape::Sawtooth];
        self.osc_levels = [1.0, 0.5, 0.3, 0.0];
        self.osc_detunes = [0.0; 4];
        self.osc_octaves = [0; 4];
        self.attack = 0.01; self.decay = 0.1; self.sustain = 0.7; self.release = 0.5;
        self.cutoff = 1000.0; self.resonance = 0.3;
        self.filter_env_amount = 0.5; self.filter_keyboard_track = 0.5;
        self.temperature = 25.0; self.power_supply_noise = 0.1; self.component_aging = 0.0;
        self.voice_mode = VoiceMode::Monophonic;
        self.portamento_speed = 0.1; self.retrigger_envelope = false;
        self.volume = 0.7; self.master_tuning = 440.0;
        self.effects = presets::default_effects();
        if let Some(first) = self.effects.first_mut() { first.enabled = true; }
        self.selected_effect = Some(0);
        self.effects_menu_open = false;
        self.mouse_pressed_note = None;
        self.keyboard_octave = 2;
    }

    /// Всплывающее окно быстрых пресетов (вызывается из верхней панели)
    fn show_presets_window(&mut self, ctx: &egui::Context) {
        let mut open = self.presets_menu_open;
        egui::Window::new(RichText::new("БЫСТРЫЕ ПРЕСЕТЫ").color(theme::ACCENT_GOLD))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.label(RichText::new("Выберите готовый пресет:")
                    .small().color(theme::TEXT_DIM));
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Моно-лид").clicked() { self.apply_preset(presets::mono_lead()); }
                    if ui.button("Поли-пэд").clicked() { self.apply_preset(presets::poly_pad()); }
                });
                ui.horizontal(|ui| {
                    if ui.button("Басс").clicked()     { self.apply_preset(presets::bass()); }
                    if ui.button("Скрипки").clicked()  { self.apply_preset(presets::strings()); }
                });
            });
        self.presets_menu_open = open;
    }
}

impl eframe::App for PolivoksApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let frame_dt = 1.0 / 60.0;
        self.check_audio_status(frame_dt);

        // ── Смена октавы компьютерной клавиатуры: Z вниз, X вверх ──
        if ctx.input(|i| i.key_pressed(egui::Key::Z)) && self.keyboard_octave > 0 {
            for (_, freq) in self.active_keys.drain(..) {
                self.synth.lock().note_off(freq);
            }
            self.keyboard_octave -= 1;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::X)) && self.keyboard_octave < 3 {
            for (_, freq) in self.active_keys.drain(..) {
                self.synth.lock().note_off(freq);
            }
            self.keyboard_octave += 1;
        }

        // ── Ввод с клавиатуры ──
        // A=C, W=C#, S=D, E=D#, D=E, F=F, T=F#, G=G, Y=G#, H=A, U=A#, J=B, K=C(окт. выше)
        let key_map = [
            (egui::Key::A, 0), (egui::Key::W, 1), (egui::Key::S, 2), (egui::Key::E, 3),
            (egui::Key::D, 4), (egui::Key::F, 5), (egui::Key::T, 6), (egui::Key::G, 7),
            (egui::Key::Y, 8), (egui::Key::H, 9), (egui::Key::U, 10), (egui::Key::J, 11),
            (egui::Key::K, 12),
        ];
        // Базовая MIDI-нота: C2 (36) + 12 * keyboard_octave
        let base_midi_visual = 36.0 + (self.keyboard_octave as f32) * 12.0;
        // Корректно учитываем master_tuning (A4)
        let base_freq = self.master_tuning * 2f32.powf((base_midi_visual - 69.0) / 12.0);

        for (key, semitone) in key_map {
            let pressed = ctx.input(|i| i.key_pressed(key));
            let released = ctx.input(|i| i.key_released(key));

            if pressed && !self.active_keys.iter().any(|(k, _)| *k == key) {
                let freq = base_freq * 2f32.powf(semitone as f32 / 12.0);
                self.synth.lock().note_on(freq);
                self.active_keys.push((key, freq));
            }
            if released {
                if let Some(idx) = self.active_keys.iter().position(|(k, _)| *k == key) {
                    let (_, freq) = self.active_keys.remove(idx);
                    self.synth.lock().note_off(freq);
                }
            }
        }

        self.update_synth_params();

        // ── Верхняя панель ────────────────────────────────────────
        egui::TopBottomPanel::top("header")
            .frame(egui::Frame::none().fill(theme::BG_PANEL).inner_margin(10.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("NEBROS RS").strong().size(20.0).color(theme::ACCENT));
                    ui.label(RichText::new("Аналоговый синтезатор")
                        .italics().size(12.0).color(theme::TEXT_MUTED));
                    ui.add_space(20.0);

                    // Индикаторы
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("AUDIO").small().color(theme::TEXT_DIM));
                        widgets::led(ui, self.audio_stream.is_some(),
                                    if self.audio_stream.is_some() { theme::ACCENT_GREEN } else { theme::ACCENT_RED });
                        ui.label(RichText::new(&self.audio_status).small().color(
                            if self.audio_status.contains("✅") { theme::ACCENT_GREEN }
                            else if self.audio_status.contains("⚠") { theme::ACCENT_ORANGE }
                            else { theme::TEXT_DIM }
                        ));
                    });

                    ui.add_space(20.0);
                    ui.label(RichText::new("MIDI").small().color(theme::TEXT_DIM));
                    widgets::led(ui, !self.active_keys.is_empty(), theme::ACCENT_GOLD);
                    if !self.active_keys.is_empty() {
                        ui.label(RichText::new(format!("{} нот", self.active_keys.len()))
                            .small().color(theme::ACCENT_GOLD));
                    }

                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Сброс").on_hover_text("Перезапустить аудио").clicked() {
                            self.manual_restart_requested = true;
                        }
                        if ui.button("Пресеты").on_hover_text("Быстрые пресеты").clicked() {
                            self.presets_menu_open = !self.presets_menu_open;
                        }
                        ui.label(RichText::new(format!("v{}", crate::consts::APP_VERSION))
                            .small().color(theme::TEXT_MUTED));
                        ui.add_space(10.0);
                        ui.label(RichText::new("громкость").small().color(theme::TEXT_DIM));
                        ui.spacing_mut().slider_width = 80.0;
                        ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).show_value(false));
                        ui.label(RichText::new(format!("{:.0}%", self.volume * 100.0))
                            .small().monospace().color(theme::TEXT));
                    });
                });
            });

        // ── Нижняя панель: экранная клавиатура ─────────────────────
        egui::TopBottomPanel::bottom("keyboard")
            .frame(egui::Frame::none().fill(theme::BG_PANEL).inner_margin(8.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🎹 КЛАВИАТУРА").small().color(theme::TEXT_DIM));
                    ui.label(RichText::new("A-W-S-E-D-F-T-G-Y-H-U-J-K  ·  октава: Z / X")
                        .small().italics().color(theme::TEXT_MUTED));
                    ui.add_space(20.0);
                    ui.label(RichText::new(format!("Октава: C{}", 2 + self.keyboard_octave))
                        .small().color(theme::ACCENT_GOLD));
                    if !self.active_keys.is_empty() {
                        ui.label(RichText::new(format!("[{}]", self.active_keys.len()))
                            .small().color(theme::ACCENT_GREEN));
                    }
                });
                ui.add_space(4.0);

                // Текущие активные ноты — из синта (единый источник истины)
                let active_freqs = self.synth.lock().active_notes();
                let active_midis: Vec<u8> = active_freqs
                    .iter()
                    .map(|f| (69.0 + 12.0 * (f / self.master_tuning).log2())
                        .round()
                        .clamp(0.0, 127.0) as u8)
                    .collect();

                let (note_on, note_off) = widgets::piano_keyboard(
                    ui,
                    &active_midis,
                    36,                   // base C2
                    4,                    // 4 октавы → C2 … C6
                    self.keyboard_octave, // где показать A..K
                    &mut self.mouse_pressed_note,
                );

                if let Some(midi) = note_on {
                    let freq = self.master_tuning * 2f32.powf((midi as f32 - 69.0) / 12.0);
                    self.synth.lock().note_on(freq);
                }
                if let Some(midi) = note_off {
                    let freq = self.master_tuning * 2f32.powf((midi as f32 - 69.0) / 12.0);
                    self.synth.lock().note_off(freq);
                }
            });

        // ── Центральная область ────────────────────────────────────
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(theme::BG_DARK).inner_margin(10.0))
            .show(ctx, |ui| {
                ui.horizontal_top(|ui| {
                    let total = ui.available_width();
                    let gap = 10.0;
                    let w1 = (total - gap * 2.0) * 0.36;
                    let w2 = (total - gap * 2.0) * 0.34;
                    let w3 = (total - gap * 2.0) * 0.30;

                    // Колонка 1 — глобальные + осцилляторы
                    ui.allocate_ui_with_layout(Vec2::new(w1, ui.available_height()),
                        Layout::top_down(egui::Align::Min), |ui| {
                            egui::ScrollArea::vertical().id_source("col1").show(ui, |ui| {
                                oscillators::render(&mut *self, ui);
                            });
                        });
                    ui.add_space(gap);

                    // Колонка 2 — фильтр + артефакты + ADSR + голосоведение/статус
                    ui.allocate_ui_with_layout(Vec2::new(w2, ui.available_height()),
                        Layout::top_down(egui::Align::Min), |ui| {
                            egui::ScrollArea::vertical().id_source("col2").show(ui, |ui| {
                                filter::render(&mut *self, ui);
                                ui.add_space(8.0);
                                envelope::render(&mut *self, ui);
                                ui.add_space(8.0);
                                voice_modes::render(&mut *self, ui);
                            });
                        });
                    ui.add_space(gap);

                    // Колонка 3 — эффекты
                    ui.allocate_ui_with_layout(Vec2::new(w3, ui.available_height()),
                        Layout::top_down(egui::Align::Min), |ui| {
                            egui::ScrollArea::vertical().id_source("col3").show(ui, |ui| {
                                effects_panel::render(&mut *self, ui);
                            });
                        });
                });
            });

        // ── Всплывающее окно пресетов ─────────────────────────────
        if self.presets_menu_open {
            self.show_presets_window(ctx);
        }

        ctx.request_repaint_after(Duration::from_millis(FRAME_DURATION_MS));
    }
}

// помогает, если поля не pub в других модулях
#[allow(dead_code)]
fn _unused(_: &Color32) { let _ = utils::format_time(0.0); }