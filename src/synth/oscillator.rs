// src/synth/oscillator.rs
//! Формы волн и их генерация

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveShape {
    Sawtooth,
    Square,
    Triangle,
}

/// Генерирует значение волны по фазе [0..1]
#[inline]
pub fn generate_waveform(shape: WaveShape, phase: f32) -> f32 {
    match shape {
        WaveShape::Sawtooth => 2.0 * phase - 1.0,
        WaveShape::Square => if phase < 0.5 { -0.8 } else { 0.8 },
        WaveShape::Triangle => {
            if phase < 0.5 {
                4.0 * phase - 1.0
            } else {
                3.0 - 4.0 * phase
            }
        }
    }
}