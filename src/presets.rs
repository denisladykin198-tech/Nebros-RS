// src/presets.rs
//! Быстрые пресеты для синтезатора

use crate::synth::{WaveShape, VoiceMode};
use crate::ui::ui_effect::{UiEffect, EffectType};

pub struct PresetData {
    pub osc_wave_shapes: [WaveShape; 4],
    pub osc_levels: [f32; 4],
    pub osc_detunes: [f32; 4],
    pub osc_octaves: [i32; 4],
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub cutoff: f32,
    pub resonance: f32,
    pub voice_mode: VoiceMode,
    pub retrigger_envelope: bool,
}

pub fn mono_lead() -> PresetData {
    PresetData {
        osc_wave_shapes: [WaveShape::Sawtooth, WaveShape::Square, WaveShape::Sawtooth, WaveShape::Sawtooth],
        osc_levels: [1.0, 0.3, 0.0, 0.0],
        osc_detunes: [0.0, 0.1, 0.0, 0.0],
        osc_octaves: [0, 0, 0, 0],
        attack: 0.01, decay: 0.1, sustain: 0.8, release: 0.2,
        cutoff: 800.0, resonance: 0.4,
        voice_mode: VoiceMode::Monophonic,
        retrigger_envelope: true,
    }
}

pub fn poly_pad() -> PresetData {
    PresetData {
        osc_wave_shapes: [WaveShape::Triangle, WaveShape::Sawtooth, WaveShape::Triangle, WaveShape::Sawtooth],
        osc_levels: [0.8, 0.6, 0.4, 0.2],
        osc_detunes: [0.0, 0.05, -0.05, 0.1],
        osc_octaves: [0, 0, 0, 0],
        attack: 0.5, decay: 1.0, sustain: 0.7, release: 3.0,
        cutoff: 500.0, resonance: 0.1,
        voice_mode: VoiceMode::Polyphonic,
        retrigger_envelope: false,
    }
}

pub fn bass() -> PresetData {
    PresetData {
        osc_wave_shapes: [WaveShape::Square, WaveShape::Sawtooth, WaveShape::Sawtooth, WaveShape::Sawtooth],
        osc_levels: [1.0, 0.5, 0.0, 0.0],
        osc_detunes: [0.0, 0.0, 0.0, 0.0],
        osc_octaves: [-2, -2, -2, -2],
        attack: 0.01, decay: 0.3, sustain: 0.9, release: 0.1,
        cutoff: 200.0, resonance: 0.0,
        voice_mode: VoiceMode::Monophonic,
        retrigger_envelope: false,
    }
}

pub fn strings() -> PresetData {
    PresetData {
        osc_wave_shapes: [WaveShape::Sawtooth, WaveShape::Sawtooth, WaveShape::Triangle, WaveShape::Triangle],
        osc_levels: [0.7, 0.7, 0.5, 0.3],
        osc_detunes: [0.0, 0.03, -0.03, 0.06],
        osc_octaves: [0, 0, 0, 0],
        attack: 0.2, decay: 0.5, sustain: 0.6, release: 1.5,
        cutoff: 1200.0, resonance: 0.2,
        voice_mode: VoiceMode::Polyphonic,
        retrigger_envelope: false,
    }
}

/// Создаёт стандартный список UI-эффектов
pub fn default_effects() -> Vec<UiEffect> {
    vec![
        UiEffect::new("Пружинный реверб", EffectType::Reverb),
        UiEffect::new("Дисторшн", EffectType::Distortion),
        UiEffect::new("Хорус", EffectType::Chorus),
        UiEffect::new("Задержка", EffectType::Delay),
        UiEffect::new("Фленджер", EffectType::Generic),
        UiEffect::new("Реверберация (цифровая)", EffectType::Generic),
        UiEffect::new("Фильтр низких частот", EffectType::Generic),
        UiEffect::new("Фильтр высоких частот", EffectType::Generic),
        UiEffect::new("Эквалайзер", EffectType::Generic),
        UiEffect::new("Компрессор", EffectType::Generic),
        UiEffect::new("Тремоло", EffectType::Generic),
        UiEffect::new("Вибрато", EffectType::Generic),
        UiEffect::new("Фейзер", EffectType::Generic),
        UiEffect::new("Битовый дрождь", EffectType::Generic),
        UiEffect::new("Резонатор", EffectType::Generic),
        UiEffect::new("Октавер", EffectType::Generic),
    ]
}

/// Дефолтные параметры реверба
pub fn default_reverb_params() -> [f32; 13] {
    [0.3, 0.25, 0.5, 0.15, 0.5, 0.5, 0.5, 0.0, 0.7, 0.1, 0.5, 0.3, 0.2]
}