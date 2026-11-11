// src/consts.rs
//! Глобальные константы синтезатора

/// Дефолтный sample rate (используется до инициализации аудио)
pub const DEFAULT_SAMPLE_RATE: f32 = 44100.0;

/// Целевой FPS для UI
pub const TARGET_FPS: f64 = 60.0;
pub const FRAME_DURATION_MS: u64 = 16;

/// Диапазоны параметров
pub const CUTOFF_RANGE: std::ops::RangeInclusive<f32> = 20.0..=8000.0;
pub const RESONANCE_RANGE: std::ops::RangeInclusive<f32> = 0.0..=0.9;
pub const VOLUME_RANGE: std::ops::RangeInclusive<f32> = 0.0..=1.0;
pub const TEMPERATURE_RANGE: std::ops::RangeInclusive<f32> = 15.0..=60.0;
pub const MASTER_TUNING_RANGE: std::ops::RangeInclusive<f32> = 430.0..=450.0;

/// ADSR диапазоны
pub const ATTACK_RANGE: std::ops::RangeInclusive<f32> = 0.001..=5.0;
pub const DECAY_RANGE: std::ops::RangeInclusive<f32> = 0.001..=5.0;
pub const SUSTAIN_RANGE: std::ops::RangeInclusive<f32> = 0.0..=1.0;
pub const RELEASE_RANGE: std::ops::RangeInclusive<f32> = 0.05..=30.0;

/// Версия приложения
pub const APP_VERSION: &str = "0.1.1";
pub const APP_TITLE: &str = "Nebros RS - Soviet Analog Synthesizer";