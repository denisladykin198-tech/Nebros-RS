// src/synth/filter.rs
//! Фильтр низких частот (state-variable)

use std::f32::consts::PI;

/// Обрабатывает один сэмпл через фильтр.
/// `state` — массив из 2 элементов (состояние фильтра), мутируется.
#[inline]
pub fn process_filter(
    input: f32,
    cutoff: f32,
    resonance: f32,
    env_mod: f32,
    keyboard_mod: f32,
    power_noise: f32,
    dt: f32,
    state: &mut [f32; 2],
) -> f32 {
    // Защита от взрывного tan(): держим fc ниже Найквиста.
    let fs = dt.recip();
    let max_fc = (fs * 0.45).max(20.0);
    let modulated_cutoff = (cutoff
        * (1.0 + env_mod)
        * (1.0 + keyboard_mod)
        * (1.0 + power_noise * 0.1))
        .clamp(20.0, max_fc);

    let g = (PI * modulated_cutoff * dt).tan();
    let k = resonance * 4.0;
    let x = input - k * state[1];

    let v0 = (x + state[0]) / (1.0 + g);
    let y0 = v0;
    let v1 = (y0 + state[1]) / (1.0 + g);
    let y1 = v1;

    state[0] = v0;
    state[1] = v1;

    (y0 * 0.3 + y1 * 0.7).tanh()
}