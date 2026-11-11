// src/synth/envelope.rs
//! ADSR-огибающая

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnvelopeStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// Вычисляет новое значение огибающей за один тик.
/// Не использует rand — детерминирована.
#[inline]
pub fn process_envelope(
    stage: EnvelopeStage,
    value: f32,
    _key_held: bool,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    _sample_rate: f32,          // оставлено для ABI, в расчёте не нужен
    dt: f32,
) -> f32 {
    let mut new_value = value;
    match stage {
        EnvelopeStage::Attack => {
            // Шаг ≤ 1.0 гарантирует, что при коротком dt
            // огибающая не «перелетит» за единицу.
            let step = (dt / attack.max(0.001)).min(1.0);
            new_value = (new_value + step).min(1.0);
        }
        EnvelopeStage::Decay => {
            let step = (dt / decay.max(0.001)).min(1.0);
            new_value = sustain + (new_value - sustain) * (1.0 - step);
            if (new_value - sustain).abs() < 0.001 {
                new_value = sustain;
            }
        }
        EnvelopeStage::Sustain => {
            new_value = sustain.clamp(0.0, 1.0);
        }
        EnvelopeStage::Release => {
            let step = (dt / release.max(0.001)).min(1.0);
            new_value *= 1.0 - step;
            if new_value < 0.00001 {
                new_value = 0.0;
            }
        }
        EnvelopeStage::Idle => {
            new_value = 0.0;
        }
    }
    new_value.clamp(0.0, 1.0)
}

/// Обновляет стадию огибающей по текущему значению
#[inline]
pub fn update_stage(
    current_stage: EnvelopeStage,
    value: f32,
    key_held: bool,
    sustain: f32,
) -> EnvelopeStage {
    match current_stage {
        EnvelopeStage::Attack => {
            if value >= 1.0 { EnvelopeStage::Decay } else { EnvelopeStage::Attack }
        }
        EnvelopeStage::Decay => {
            if value <= sustain {
                if key_held { EnvelopeStage::Sustain } else { EnvelopeStage::Release }
            } else {
                EnvelopeStage::Decay
            }
        }
        EnvelopeStage::Sustain => {
            if key_held { EnvelopeStage::Sustain } else { EnvelopeStage::Release }
        }
        EnvelopeStage::Release => {
            if value <= 0.001 { EnvelopeStage::Idle } else { EnvelopeStage::Release }
        }
        EnvelopeStage::Idle => EnvelopeStage::Idle,
    }
}