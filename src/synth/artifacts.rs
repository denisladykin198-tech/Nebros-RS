// src/synth/artifacts.rs
// Артефакты: температура, просадки напряжения, шумы

use rand::rngs::SmallRng;
use rand::{Rng};
use std::f32::consts::PI;

use super::VoiceMode;

/// Просадка напряжения (Voltage Sag)
pub struct VoltageSag {
    active: bool,
    duration: f32,
    progress: f32,
    strength: f32,
    frequency_multiplier: f32,
}

impl VoltageSag {
    pub fn new() -> Self {
        Self {
            active: false,
            duration: 0.0,
            progress: 0.0,
            strength: 0.0,
            frequency_multiplier: 1.0,
        }
    }

    /// Обновляет состояние просадки.
    /// Использует `rng` вместо `rand::random()` — в 10-50 раз быстрее.
    pub fn update(&mut self, dt: f32, power_noise: f32, voice_mode: VoiceMode, rng: &mut SmallRng) -> f32 {
        let mut multiplier = 1.0;
        self.frequency_multiplier = match voice_mode {
            VoiceMode::Polyphonic => 1.5,
            _ => 1.0,
        };

        if self.active {
            self.progress += dt / self.duration;
            if self.progress >= 1.0 {
                self.active = false;
                multiplier = 1.0;
            } else {
                let t = self.progress;
                if t < 0.2 {
                    multiplier = 1.0 - self.strength * (t / 0.2).powf(2.0);
                } else {
                    let recovery = (t - 0.2) / 0.8;
                    multiplier = 1.0 - self.strength * (1.0 - recovery).powf(0.5);
                }
            }
        } else {
            let p: f32 = rng.gen();
            if p < 0.000007 * power_noise * self.frequency_multiplier {
                self.active = true;
                self.duration = 0.03 + rng.gen::<f32>() * 0.1;
                self.progress = 0.0;
                self.strength = 0.5 + rng.gen::<f32>() * 0.4;
            }
        }
        multiplier.clamp(0.15, 1.0)
    }
}

/// Добавляет все артефакты к сигналу.
pub fn add_artifacts(
    signal: f32,
    time: f32,
    temperature: f32,
    power_noise: f32,
    component_aging: f32,
    voice_mode: VoiceMode,
    rng: &mut SmallRng,
) -> f32 {
    let mut result = signal;

    // Старение компонентов
    result *= 1.0 - component_aging * 0.2;

    // Фон 50 Гц и гармоники
    let t2pi = time * 2.0 * PI;
    result += (t2pi * 50.0).sin() * power_noise * 0.15;
    result += (t2pi * 100.0).sin() * power_noise * 0.08;
    result += (t2pi * 150.0).sin() * power_noise * 0.04;
    result += (t2pi * 250.0).sin() * power_noise * 0.02;

    // Тепловой шум и шипение
    let thermal: f32 = rng.gen();
    result += (thermal - 0.5) * (temperature * 0.00025) * power_noise;

    let hiss: f32 = rng.gen();
    result += (hiss - 0.5) * (power_noise * 0.05) * (1.0 + (temperature - 25.0) * 0.01);

    // Рябь питания
    result += ((t2pi * 100.0 + 0.5).sin().abs()) * power_noise * 0.1;

    // ТВ-наводки
    result += (time * 15625.0 * 2.0 * PI).sin()
        * power_noise * 0.02
        * (0.5 + 0.5 * (time * 0.5).sin());

    // Импульсные помехи
    let imp_p: f32 = rng.gen();
    if imp_p < 0.0001 * power_noise {
        result += (time * 2000.0 * PI).sin().max(0.0) * 0.4 * power_noise;
    }

    // Клики
    let click_p: f32 = rng.gen();
    let click_probability = match voice_mode {
        VoiceMode::Polyphonic => 0.0001,
        _ => 0.0002,
    };
    if click_p < click_probability * power_noise {
        let strength = 0.6 + power_noise * 0.6;
        let sign: f32 = rng.gen();
        result += if sign > 0.5 { strength } else { -strength };
    }

    // Дребезг старых компонентов
    let crackle_p: f32 = rng.gen();
    if crackle_p < 0.0001 * component_aging {
        let c: f32 = rng.gen();
        result += (c - 0.5) * 0.3 * component_aging;
    }

    result.clamp(-1.0, 1.0)
}