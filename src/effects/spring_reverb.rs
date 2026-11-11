//! Пружинный ревербератор
//! Стерео-обработка, оптимизированный шум, безопасная дробная задержка, защита от денормализации

use std::f32::consts::PI;
use super::Effect;

// Константа для защиты от денормализованных чисел (стандарт индустрии)
const DENORMAL_FLT: f32 = 1e-15;

pub struct SovietSpringReverb {
    // === UI-параметры (сырые значения 0.0-1.0) ===
    mix: f32,
    decay_ui: f32,
    damping: f32,
    pre_delay_ui: f32,
    room_size_ui: f32,
    diffusion: f32,
    brightness: f32,
    modulation: f32,
    stereo_width: f32,
    spring_noise_ui: f32,
    spring_tension: f32,
    metal_fatigue_ui: f32,
    rust_level_ui: f32,

    // === Реальные вычисленные значения ===
    decay: f32,
    pre_delay: f32,
    room_size: f32,

    // === Структура фильтров ===
    comb_filters: [CombFilter; 4],
    comb_filters_r: [CombFilter; 4],
    allpass_filters: [AllpassFilter; 2],
    allpass_filters_r: [AllpassFilter; 2],

    // === Буферы ===
    sample_rate: f32,
    pre_delay_buffer: Vec<f32>,
    pre_delay_buffer_r: Vec<f32>,
    pre_delay_pos: usize,

    // === Фильтры ===
    lowpass: OnePoleLowpass,
    lowpass_r: OnePoleLowpass,
    highpass: OnePoleHighpass,
    highpass_r: OnePoleHighpass,

    // === Модуляция и время ===
    lfo_phase: f32,
    lfo_rate: f32,
    time: f32,

    // === Быстрый LCG для шума (замена медленному SmallRng на сэмпл) ===
    noise_seed: u32,
    
    // === Огибающая для артефактов (чтобы избежать кликов) ===
    artifact_env: f32,
}

impl SovietSpringReverb {
    pub fn new(sample_rate: f32) -> Self {
        // Базовые времена в секундах
        let comb_times = [0.0297, 0.0371, 0.0411, 0.0437];
        let sr = sample_rate;

        // Левый канал
        let comb_filters = [
            CombFilter::new((comb_times[0] * sr) as usize, 0.5, 0.3, 0.8),
            CombFilter::new((comb_times[1] * sr) as usize, 0.5, 0.3, 1.1),
            CombFilter::new((comb_times[2] * sr) as usize, 0.5, 0.3, 0.9),
            CombFilter::new((comb_times[3] * sr) as usize, 0.5, 0.3, 1.2),
        ];

        // Правый канал (микро-смещения для стерео-ширины)
        let comb_filters_r = [
            CombFilter::new(((comb_times[0] * 1.03) * sr) as usize, 0.5, 0.3, 1.3),
            CombFilter::new(((comb_times[1] * 0.97) * sr) as usize, 0.5, 0.3, 0.7),
            CombFilter::new(((comb_times[2] * 1.02) * sr) as usize, 0.5, 0.3, 1.0),
            CombFilter::new(((comb_times[3] * 0.98) * sr) as usize, 0.5, 0.3, 0.6),
        ];

        let allpass_filters = [
            AllpassFilter::new((0.005 * sr) as usize, 0.5),
            AllpassFilter::new((0.0017 * sr) as usize, 0.5),
        ];
        let allpass_filters_r = [
            AllpassFilter::new(((0.005 * 0.95) * sr) as usize, 0.5),
            AllpassFilter::new(((0.0017 * 1.05) * sr) as usize, 0.5),
        ];

        let pre_delay_samples = ((0.015 * sr) as usize).max(1);

        Self {
            mix: 0.3,
            decay_ui: 0.2,
            damping: 0.4,
            pre_delay_ui: 0.075,
            room_size_ui: 0.333,
            diffusion: 0.5,
            brightness: 0.5,
            modulation: 0.0,
            stereo_width: 0.7,
            spring_noise_ui: 0.1,
            spring_tension: 0.5,
            metal_fatigue_ui: 0.3,
            rust_level_ui: 0.2,
            decay: 2.0,
            pre_delay: 0.015,
            room_size: 1.0,
            comb_filters,
            comb_filters_r,
            allpass_filters,
            allpass_filters_r,
            sample_rate: sr,
            pre_delay_buffer: vec![0.0; pre_delay_samples],
            pre_delay_buffer_r: vec![0.0; pre_delay_samples],
            pre_delay_pos: 0,
            lowpass: OnePoleLowpass::new(10000.0, sr),
            lowpass_r: OnePoleLowpass::new(10000.0, sr),
            highpass: OnePoleHighpass::new(80.0, sr),
            highpass_r: OnePoleHighpass::new(80.0, sr),
            lfo_phase: 0.0,
            lfo_rate: 0.5,
            time: 0.0,
            noise_seed: 0x50564542,
            artifact_env: 0.0,
        }
    }

    // ============================================
    // СЕТТЕРЫ
    // ============================================

    pub fn set_mix(&mut self, mix: f32) { self.mix = mix.clamp(0.0, 1.0); }

    pub fn set_decay(&mut self, decay: f32) {
        self.decay = decay.clamp(0.1, 10.0);
        self.decay_ui = (self.decay - 0.1) / 9.9;
        self.update_comb_feedback();
    }

    pub fn set_damping(&mut self, damping: f32) {
        self.damping = damping.clamp(0.0, 1.0);
        let cutoff = 8000.0 * (1.0 - self.damping * 0.8);
        self.lowpass.set_cutoff(cutoff, self.sample_rate);
        self.lowpass_r.set_cutoff(cutoff * 0.95, self.sample_rate);
    }

    pub fn set_diffusion(&mut self, diffusion: f32) {
        self.diffusion = diffusion.clamp(0.0, 1.0);
        let gain = 0.2 + self.diffusion * 0.3;
        for i in 0..2 {
            self.allpass_filters[i].set_gain(gain);
            self.allpass_filters_r[i].set_gain(gain * 0.9);
        }
    }

    pub fn set_pre_delay(&mut self, pre_delay: f32) {
        self.pre_delay = pre_delay.clamp(0.0, 0.2);
        self.pre_delay_ui = self.pre_delay / 0.2;
        let samples = ((self.pre_delay * self.sample_rate) as usize).max(1);
        if samples != self.pre_delay_buffer.len() {
            self.pre_delay_buffer = vec![0.0; samples];
            self.pre_delay_buffer_r = vec![0.0; samples];
            self.pre_delay_pos = 0;
        }
    }

    pub fn set_room_size(&mut self, room_size: f32) {
        self.room_size = room_size.clamp(0.5, 3.0);
        self.room_size_ui = (self.room_size - 0.5) / 2.5;
        self.update_comb_feedback();
    }

    pub fn set_brightness(&mut self, brightness: f32) {
        self.brightness = brightness.clamp(0.0, 1.0);
        // Яркость управляет и highpass (убирает гул), и lowpass (пропускает верх)
        let high_cut = 50.0 + self.brightness.powf(2.0) * 3950.0;
        self.highpass.set_cutoff(high_cut, self.sample_rate);
        self.highpass_r.set_cutoff(high_cut * 1.1, self.sample_rate);
        
        let damp_factor = 0.5 + self.brightness * 0.5;
        self.lowpass.set_cutoff(12000.0 * damp_factor, self.sample_rate);
        self.lowpass_r.set_cutoff(12000.0 * damp_factor * 0.9, self.sample_rate);
    }

    pub fn set_modulation(&mut self, modulation: f32) {
        self.modulation = modulation.clamp(0.0, 1.0);
        self.lfo_rate = 0.2 + self.modulation * 5.0;
        let mod_amt = self.modulation * 0.02; // Макс 2% модуляции задержки (безопасно)
        for comb in &mut self.comb_filters { comb.set_modulation(mod_amt); }
        for comb in &mut self.comb_filters_r { comb.set_modulation(mod_amt * 0.8); }
    }

    pub fn set_stereo_width(&mut self, width: f32) {
        self.stereo_width = width.clamp(0.0, 1.5); // Разрешаем super-wide до 1.5
    }

    pub fn set_spring_noise(&mut self, noise: f32) { self.spring_noise_ui = noise.clamp(0.0, 1.0); }
    pub fn set_spring_tension(&mut self, tension: f32) {
        self.spring_tension = tension.clamp(0.0, 1.0);
        self.update_comb_feedback(); // Пересчитывает feedback с учетом натяжения
    }
    pub fn set_metal_fatigue(&mut self, fatigue: f32) { self.metal_fatigue_ui = fatigue.clamp(0.0, 1.0); }
    pub fn set_rust_level(&mut self, rust: f32) {
        self.rust_level_ui = rust.clamp(0.0, 1.0);
        self.update_comb_feedback();
    }

    fn decay_to_feedback(decay: f32, room_size: f32, tension: f32, rust: f32) -> f32 {
        const AVG_COMB_DELAY: f32 = 0.0379;
        // Ржавчина уменьшает эффективное время затухания
        let rust_factor = 1.0 - rust * 0.3;
        // Натяжение немного увеличивает добротность
        let tension_factor = 1.0 + (tension - 0.5) * 0.2;
        
        let effective_decay = decay * room_size.powf(1.2) * rust_factor * tension_factor;
        let g = 10f32.powf(-3.0 * AVG_COMB_DELAY / effective_decay.max(0.1));
        g.min(0.985) // Жесткий лимит для стабильности
    }

    fn update_comb_feedback(&mut self) {
        let scaled_feedback = Self::decay_to_feedback(
            self.decay, self.room_size, self.spring_tension, self.rust_level_ui
        );
        for i in 0..4 {
            self.comb_filters[i].set_feedback(scaled_feedback);
            self.comb_filters_r[i].set_feedback((scaled_feedback * 0.95).min(0.985));
        }
    }

    // ============================================
    // ГЛАВНЫЙ ПРОЦЕССИНГ
    // ============================================

    #[inline]
    pub fn process_stereo(&mut self, input: f32, dt: f32) -> (f32, f32) {
        self.time += dt;

        // 1. LFO для модуляции (Wow/Flutter эффект пружины)
        self.lfo_phase += 2.0 * PI * self.lfo_rate * dt;
        if self.lfo_phase > 2.0 * PI { self.lfo_phase -= 2.0 * PI; }
        let lfo_tri = (2.0 * (self.lfo_phase / (2.0 * PI)).abs() - 1.0) * self.modulation * 0.015;

        // 2. Быстрый псевдо-шум (LCG) вместо тяжелого SmallRng на каждый сэмпл
        self.noise_seed = self.noise_seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let fast_noise = (self.noise_seed as f32 / u32::MAX as f32) - 0.5;
        
        // Непрерывный гул пружины + редкие всплески
        let noise_level = self.spring_noise_ui.powf(1.5);
        let spring_noise = fast_noise * noise_level * 0.02 + 
                          (if fast_noise > 0.45 { fast_noise * noise_level * 0.1 } else { 0.0 });

        // 3. Предфильтрация
        let modulated_input = input * (1.0 + lfo_tri);
        let filtered_input = self.highpass.process(modulated_input + spring_noise);

        // 4. Предзадержка
        let (left, right) = self.process_pre_delay(filtered_input);

        // 5. Комб-фильтры (с дробной задержкой внутри)
        let mut comb_left = 0.0;
        for (i, comb) in self.comb_filters.iter_mut().enumerate() {
            comb_left += comb.process(left * (0.7 + 0.3 * (i as f32 / 4.0)), dt);
        }
        comb_left /= 4.0;

        let mut comb_right = 0.0;
        for (i, comb) in self.comb_filters_r.iter_mut().enumerate() {
            comb_right += comb.process(right * (0.7 + 0.3 * ((4 - i) as f32 / 4.0)), dt);
        }
        comb_right /= 4.0;

        // 6. Аллпасс-фильтры (диффузия)
        let mut allpass_left = comb_left;
        for allpass in &mut self.allpass_filters { allpass_left = allpass.process(allpass_left); }
        
        let mut allpass_right = comb_right;
        for allpass in &mut self.allpass_filters_r { allpass_right = allpass.process(allpass_right); }

        // 7. Постфильтрация
        let mut wet_left = self.lowpass.process(allpass_left);
        let mut wet_right = self.lowpass_r.process(allpass_right);

        // 8. Артефакты (Усталость металла и ржавчина)
        // Используем огибающую для сглаживания щелчков, а не мгновенный скачок
        self.artifact_env *= 0.95; // Затухание огибающей
        
        let fatigue = self.metal_fatigue_ui.powf(1.5);
        if fatigue > 0.01 && fast_noise > (1.0 - fatigue * 0.01) {
            self.artifact_env = fatigue * 0.15; // Запуск огибающей щелчка
        }
        
        let rust = self.rust_level_ui.powf(2.0);
        let rust_hiss = fast_noise * rust * 0.01;
        
        let artifact_signal = (self.artifact_env + rust_hiss) * (fast_noise * 2.0); // Простой шум для треска

        wet_left += artifact_signal * 0.7;
        wet_right += artifact_signal * 0.3;

        // 9. Стерео обработка (Mid/Side матрица - стандарт индустрии)
        let mid = (wet_left + wet_right) * 0.5;
        let side = (wet_left - wet_right) * 0.5;
        let width = self.stereo_width;
        
        let final_left = mid + side * width;
        let final_right = mid - side * width;

        // 10. Soft-clip (tanh) для предотвращения цифрового клиппинга
        let clipped_left = final_left.tanh();
        let clipped_right = final_right.tanh();

        // 11. Микширование
        let dry = input;
        (
            dry * (1.0 - self.mix) + clipped_left * self.mix,
            dry * (1.0 - self.mix) + clipped_right * self.mix,
        )
    }

    #[inline]
    fn process_pre_delay(&mut self, input: f32) -> (f32, f32) {
        if self.pre_delay_buffer.is_empty() { return (input, input); }
        let left = self.pre_delay_buffer[self.pre_delay_pos];
        let right = self.pre_delay_buffer_r[self.pre_delay_pos];
        self.pre_delay_buffer[self.pre_delay_pos] = input;
        self.pre_delay_buffer_r[self.pre_delay_pos] = input * 0.9; // Легкий Haas-эффект
        self.pre_delay_pos = (self.pre_delay_pos + 1) % self.pre_delay_buffer.len();
        (left, right)
    }

    pub fn hit_cabinet(&mut self, strength: f32) {
        let strength = strength.clamp(0.0, 1.0);
        // Плавный ввод импульса вместо резкого скачка
        self.artifact_env = (self.artifact_env + strength * 0.5).min(1.0);
        self.spring_noise_ui = (self.spring_noise_ui + strength * 0.3).min(1.0);
    }

    pub fn clean_contacts(&mut self) {
        for comb in &mut self.comb_filters { comb.reset(); }
        for comb in &mut self.comb_filters_r { comb.reset(); }
        for allpass in &mut self.allpass_filters { allpass.reset(); }
        for allpass in &mut self.allpass_filters_r { allpass.reset(); }
        self.pre_delay_buffer.fill(0.0);
        self.pre_delay_buffer_r.fill(0.0);
        self.pre_delay_pos = 0;
        self.spring_noise_ui = 0.1;
        self.rust_level_ui = 0.2;
        self.artifact_env = 0.0;
    }
}

// ============================================
// TRAIT IMPL
// ============================================

impl Effect for SovietSpringReverb {
    #[inline]
    fn process(&mut self, input: f32, dt: f32) -> f32 {
        let (l, r) = self.process_stereo(input, dt);
        (l + r) * 0.5 // Моно-совместимость
    }

    fn set_parameter(&mut self, param: usize, value: f32) {
        let v = value.clamp(0.0, 1.0);
        match param {
            0 => self.set_mix(v),
            1 => self.set_decay(v * 9.9 + 0.1),
            2 => self.set_damping(v),
            3 => self.set_pre_delay(v * 0.2),
            4 => self.set_room_size(v * 2.5 + 0.5),
            5 => self.set_diffusion(v),
            6 => self.set_brightness(v),
            7 => self.set_modulation(v),
            8 => self.set_stereo_width(v * 1.5), // UI 0-1 maps to 0-1.5 width
            9 => self.set_spring_noise(v),
            10 => self.set_spring_tension(v),
            11 => self.set_metal_fatigue(v),
            12 => self.set_rust_level(v),
            _ => {}
        }
    }

    fn get_parameter(&self, param: usize) -> f32 {
        match param {
            0 => self.mix, 1 => self.decay_ui, 2 => self.damping, 3 => self.pre_delay_ui,
            4 => self.room_size_ui, 5 => self.diffusion, 6 => self.brightness, 7 => self.modulation,
            8 => self.stereo_width / 1.5, 9 => self.spring_noise_ui, 10 => self.spring_tension,
            11 => self.metal_fatigue_ui, 12 => self.rust_level_ui, _ => 0.0,
        }
    }

    fn get_parameter_name(&self, param: usize) -> &'static str {
        match param {
            0 => "Mix", 1 => "Decay", 2 => "Damping", 3 => "Pre Delay", 4 => "Room Size",
            5 => "Diffusion", 6 => "Brightness", 7 => "Modulation", 8 => "Stereo Width",
            9 => "Spring Noise", 10 => "Spring Tension", 11 => "Metal Fatigue", 12 => "Rust Level",
            _ => "Unknown",
        }
    }

    fn get_name(&self) -> &'static str { "Soviet Spring Reverb" }
}

// ============================================
// COMB FILTER (с безопасной дробной задержкой)
// ============================================

struct CombFilter {
    buffer: Vec<f32>,
    write_pos: usize,
    base_delay_samples: usize,
    max_delay_samples: usize, // Для безопасного чтения при модуляции
    feedback: f32,
    filter_state: f32,
    damping: f32,
    modulation_amt: f32,
    lfo_phase: f32,
    lfo_rate: f32,
}

impl CombFilter {
    fn new(delay_samples: usize, feedback: f32, damping: f32, lfo_rate: f32) -> Self {
        // Добавляем запас (headroom) для модуляции задержки + 2 сэмпла для линейной интерполяции
        let max_delay = (delay_samples as f32 * 1.1) as usize + 2;
        Self {
            buffer: vec![0.0; max_delay.max(2)],
            write_pos: 0,
            base_delay_samples: delay_samples.max(2),
            max_delay_samples: max_delay,
            feedback: feedback.clamp(0.0, 0.985),
            filter_state: 0.0,
            damping: damping.clamp(0.0, 0.5),
            modulation_amt: 0.0,
            lfo_phase: 0.0,
            lfo_rate,
        }
    }

    fn set_feedback(&mut self, feedback: f32) { self.feedback = feedback.clamp(0.0, 0.985); }
    fn set_modulation(&mut self, amount: f32) { self.modulation_amt = amount.clamp(0.0, 0.05); }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
        self.filter_state = 0.0;
        self.lfo_phase = 0.0;
    }

    #[inline]
    fn process(&mut self, input: f32, dt: f32) -> f32 {
        // Защита от денормализации
        let safe_input = input + DENORMAL_FLT;

        let mut delayed = if self.modulation_amt > 0.0 {
            self.lfo_phase += 2.0 * PI * self.lfo_rate * dt;
            if self.lfo_phase > 2.0 * PI { self.lfo_phase -= 2.0 * PI; }

            let mod_value = (self.lfo_phase).sin() * self.modulation_amt;
            let delay_f = self.base_delay_samples as f32 * (1.0 + mod_value);
            let delay_int = delay_f.floor() as usize;
            let delay_frac = delay_f - delay_int as f32;

            // Безопасное чтение с проверкой границ
            let d = delay_int.min(self.max_delay_samples - 2).max(1);
            let read_pos_0 = (self.write_pos + self.buffer.len() - d) % self.buffer.len();
            let read_pos_1 = (read_pos_0 + self.buffer.len() - 1) % self.buffer.len();

            // Линейная интерполяция
            self.buffer[read_pos_0] * (1.0 - delay_frac) + self.buffer[read_pos_1] * delay_frac
        } else {
            let read_pos = (self.write_pos + self.buffer.len() - self.base_delay_samples) % self.buffer.len();
            self.buffer[read_pos]
        };

        // Демпфирование (Lowpass в цепи обратной связи)
        self.filter_state += self.damping * (delayed - self.filter_state);
        delayed = self.filter_state + DENORMAL_FLT;

        // Запись в буфер
        self.buffer[self.write_pos] = safe_input + delayed * self.feedback;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();

        delayed
    }
}

// ============================================
// ALLPASS FILTER (Стандартная формула Шредера)
// ============================================

struct AllpassFilter {
    buffer: Vec<f32>,
    write_pos: usize,
    delay_samples: usize,
    gain: f32,
}

impl AllpassFilter {
    fn new(delay_samples: usize, gain: f32) -> Self {
        Self {
            buffer: vec![0.0; delay_samples.max(1)],
            write_pos: 0,
            delay_samples: delay_samples.max(1),
            gain: gain.clamp(0.0, 0.95),
        }
    }

    fn set_gain(&mut self, gain: f32) { self.gain = gain.clamp(0.0, 0.95); }
    fn reset(&mut self) { self.buffer.fill(0.0); self.write_pos = 0; }

    #[inline]
    fn process(&mut self, input: f32) -> f32 {
        let safe_in = input + DENORMAL_FLT;
        let read_pos = (self.write_pos + self.buffer.len() - self.delay_samples) % self.buffer.len();
        let delayed = self.buffer[read_pos] + DENORMAL_FLT;

        let output = -self.gain * safe_in + delayed;
        self.buffer[self.write_pos] = safe_in + self.gain * delayed;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
        output
    }
}

// ============================================
// ONE-POLE FILTERS
// ============================================

struct OnePoleLowpass { last_out: f32, alpha: f32 }
impl OnePoleLowpass {
    fn new(cutoff: f32, sample_rate: f32) -> Self {
        let rc = 1.0 / (2.0 * PI * cutoff.max(1.0));
        let alpha = (1.0 / sample_rate) / (rc + (1.0 / sample_rate));
        Self { last_out: 0.0, alpha: alpha.clamp(0.0, 1.0) }
    }
    fn set_cutoff(&mut self, cutoff: f32, sample_rate: f32) {
        let rc = 1.0 / (2.0 * PI * cutoff.max(1.0));
        self.alpha = ((1.0 / sample_rate) / (rc + (1.0 / sample_rate))).clamp(0.0, 1.0);
    }
    #[inline]
    fn process(&mut self, input: f32) -> f32 {
        let safe_in = input + DENORMAL_FLT;
        self.last_out += self.alpha * (safe_in - self.last_out);
        self.last_out + DENORMAL_FLT
    }
}

struct OnePoleHighpass { last_in: f32, last_out: f32, alpha: f32 }
impl OnePoleHighpass {
    fn new(cutoff: f32, sample_rate: f32) -> Self {
        let rc = 1.0 / (2.0 * PI * cutoff.max(1.0));
        let alpha = rc / (rc + (1.0 / sample_rate));
        Self { last_in: 0.0, last_out: 0.0, alpha: alpha.clamp(0.0, 1.0) }
    }
    fn set_cutoff(&mut self, cutoff: f32, sample_rate: f32) {
        let rc = 1.0 / (2.0 * PI * cutoff.max(1.0));
        self.alpha = (rc / (rc + (1.0 / sample_rate))).clamp(0.0, 1.0);
    }
    #[inline]
    fn process(&mut self, input: f32) -> f32 {
        let safe_in = input + DENORMAL_FLT;
        self.last_out = self.alpha * (self.last_out + safe_in - self.last_in);
        self.last_in = safe_in;
        self.last_out + DENORMAL_FLT
    }
}