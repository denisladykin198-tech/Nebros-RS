// src/synth/mod.rs
//! Основной модуль синтезатора

pub mod voice;
pub mod oscillator;
pub mod envelope;
pub mod filter;
pub mod artifacts;

use rand::rngs::SmallRng;
use rand::SeedableRng;

use crate::effects::spring_reverb::SovietSpringReverb;
use crate::effects::Effect; 
use crate::consts::DEFAULT_SAMPLE_RATE;

use voice::Voice;
pub use oscillator::{WaveShape, generate_waveform};
use envelope::{EnvelopeStage, process_envelope, update_stage};
use artifacts::{VoltageSag, add_artifacts};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceMode {
    Monophonic,
    Legato,
    Polyphonic,
    Portamento,
}

pub use oscillator::WaveShape as WaveShapePublic;

pub struct PolivoksSynth {
    osc_wave_shapes: [WaveShape; 4],
    osc_levels: [f32; 4],
    osc_detunes: [f32; 4],
    osc_octaves: [i32; 4],

    filter_cutoff: f32,
    filter_resonance: f32,
    filter_state: [f32; 2],
    filter_env_amount: f32,
    filter_keyboard_track: f32,

    envelope_attack: f32,
    envelope_decay: f32,
    envelope_sustain: f32,
    envelope_release: f32,

    voice_mode: VoiceMode,
    monophonic_note: Option<f32>,
    monophonic_phase: f32,
    monophonic_osc_phases: [f32; 4],
    monophonic_envelope: f32,
    monophonic_stage: EnvelopeStage,
    monophonic_key_held: bool,
    legato_active: bool,
    legato_note_stack: Vec<f32>,
    portamento_speed: f32,
    current_frequency: f32,
    target_frequency: f32,

    polyphonic_voices: [Voice; 8],
    voice_stealing_index: usize,

    temperature: f32,
    power_noise: f32,
    component_aging: f32,

    pub time: f32,
    pub sample_rate: f32,
    voltage_sag: VoltageSag,

    retrigger_envelope: bool,
    master_tuning: f32,
    volume: f32,

    reverb: SovietSpringReverb,
    reverb_enabled: bool,
    reverb_params: [f32; 13],

    rng: SmallRng,
}

impl PolivoksSynth {
    pub fn new() -> Self {
        let polyphonic_voices = [
            Voice::new(), Voice::new(), Voice::new(), Voice::new(),
            Voice::new(), Voice::new(), Voice::new(), Voice::new(),
        ];
        let sample_rate = DEFAULT_SAMPLE_RATE;

        Self {
            osc_wave_shapes: [WaveShape::Sawtooth; 4],
            osc_levels: [0.25; 4],
            osc_detunes: [0.0; 4],
            osc_octaves: [0; 4],

            filter_cutoff: 1000.0,
            filter_resonance: 0.3,
            filter_state: [0.0; 2],
            filter_env_amount: 0.5,
            filter_keyboard_track: 0.5,

            envelope_attack: 0.01,
            envelope_decay: 0.1,
            envelope_sustain: 0.7,
            envelope_release: 0.5,

            voice_mode: VoiceMode::Monophonic,
            monophonic_note: None,
            monophonic_phase: 0.0,
            monophonic_osc_phases: [0.0; 4],
            monophonic_envelope: 0.0,
            monophonic_stage: EnvelopeStage::Idle,
            monophonic_key_held: false,
            legato_active: false,
            legato_note_stack: Vec::new(),
            portamento_speed: 0.1,
            current_frequency: 0.0,
            target_frequency: 0.0,

            polyphonic_voices,
            voice_stealing_index: 0,

            temperature: 25.0,
            power_noise: 0.1,
            component_aging: 0.0,

            time: 0.0,
            sample_rate,
            voltage_sag: VoltageSag::new(),

            retrigger_envelope: false,
            master_tuning: 440.0,
            volume: 1.0,

            reverb: SovietSpringReverb::new(sample_rate),
            reverb_enabled: false,
            reverb_params: [
                0.3, 0.25, 0.5, 0.15, 0.5, 0.5, 0.5, 0.0, 0.7, 0.1, 0.5, 0.3, 0.2,
            ],

            rng: SmallRng::seed_from_u64(0xC0FFEE_BABE),
        }
    }

    // === Осцилляторы ===
    pub fn set_osc_wave_shape(&mut self, index: usize, shape: WaveShape) {
        if index < 4 { self.osc_wave_shapes[index] = shape; }
    }
    pub fn set_osc_level(&mut self, index: usize, level: f32) {
        if index < 4 { self.osc_levels[index] = level.clamp(0.0, 1.0); }
    }
    pub fn set_osc_detune(&mut self, index: usize, detune: f32) {
        if index < 4 { self.osc_detunes[index] = detune.clamp(-1.0, 1.0); }
    }
    pub fn set_osc_octave(&mut self, index: usize, octave: i32) {
        if index < 4 { self.osc_octaves[index] = octave.clamp(-2, 2); }
    }

    pub fn set_cutoff(&mut self, cutoff: f32) { self.filter_cutoff = cutoff; }
    pub fn set_resonance(&mut self, resonance: f32) { self.filter_resonance = resonance; }
    pub fn set_filter_env_amount(&mut self, amount: f32) { self.filter_env_amount = amount; }
    pub fn set_filter_keyboard_track(&mut self, track: f32) { self.filter_keyboard_track = track; }

    pub fn set_adsr(&mut self, attack: f32, decay: f32, sustain: f32, release: f32) {
        self.envelope_attack = attack.clamp(0.001, 5.0);
        self.envelope_decay = decay.clamp(0.001, 5.0);
        self.envelope_sustain = sustain.clamp(0.0, 1.0);
        self.envelope_release = release.clamp(0.05, 30.0);
    }

    pub fn set_temperature(&mut self, temp: f32) { self.temperature = temp; }
    pub fn set_power_noise(&mut self, noise: f32) { self.power_noise = noise; }
    pub fn set_component_aging(&mut self, aging: f32) { self.component_aging = aging; }
    pub fn set_master_tuning(&mut self, tuning: f32) { self.master_tuning = tuning; }
    pub fn set_volume(&mut self, volume: f32) { self.volume = volume.clamp(0.0, 1.0); }

    pub fn set_voice_mode(&mut self, mode: VoiceMode) {
        if self.voice_mode == mode { return; }
        self.voice_mode = mode;
        match mode {
            VoiceMode::Monophonic => { self.legato_note_stack.clear(); }
            VoiceMode::Legato => {
                self.legato_active = self.monophonic_note.is_some();
                if let Some(note) = self.monophonic_note {
                    if !self.legato_note_stack.contains(&note) {
                        self.legato_note_stack.push(note);
                    }
                }
            }
            VoiceMode::Polyphonic => {
                for v in &mut self.polyphonic_voices { v.reset(); }
                self.monophonic_note = None;
                self.monophonic_envelope = 0.0;
                self.monophonic_stage = EnvelopeStage::Idle;
                self.monophonic_key_held = false;
                self.legato_note_stack.clear();
                self.current_frequency = 0.0;
                self.target_frequency = 0.0;
            }
            VoiceMode::Portamento => {
                self.legato_active = self.monophonic_note.is_some();
                if let Some(note) = self.monophonic_note {
                    self.current_frequency = note;
                    self.target_frequency = note;
                    if !self.legato_note_stack.contains(&note) {
                        self.legato_note_stack.push(note);
                    }
                }
            }
        }
    }

    pub fn set_portamento_speed(&mut self, speed: f32) {
        self.portamento_speed = speed.clamp(0.001, 2.0);
    }
    pub fn portamento_speed(&self) -> f32 { self.portamento_speed }
    pub fn legato_stack_size(&self) -> usize { self.legato_note_stack.len() }
    pub fn set_retrigger_envelope(&mut self, retrigger: bool) { self.retrigger_envelope = retrigger; }
    pub fn retrigger_envelope(&self) -> bool { self.retrigger_envelope }
    pub fn voice_mode(&self) -> VoiceMode { self.voice_mode }
    pub fn current_envelope(&self) -> f32 { self.monophonic_envelope }

    pub fn set_reverb_enabled(&mut self, enabled: bool) {
        self.reverb_enabled = enabled;
    }
    pub fn reverb_enabled(&self) -> bool { self.reverb_enabled }

    pub fn set_reverb_parameter(&mut self, param: usize, value: f32) {
        if param < self.reverb_params.len() {
            let clamped = value.clamp(0.0, 1.0);
            self.reverb_params[param] = clamped;
            self.reverb.set_parameter(param, clamped);
        }
    }
    pub fn reverb_parameter(&self, param: usize) -> f32 {
        self.reverb_params.get(param).copied().unwrap_or(0.0)
    }
    pub fn hit_reverb_cabinet(&mut self, strength: f32) { self.reverb.hit_cabinet(strength); }
    pub fn clean_reverb_contacts(&mut self) { self.reverb.clean_contacts(); }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.reverb = SovietSpringReverb::new(sample_rate);
        for i in 0..self.reverb_params.len() {
            self.reverb.set_parameter(i, self.reverb_params[i]);
        }
    }

    // === Note On / Off ===
    pub fn note_on(&mut self, freq: f32) {
        log::debug!("note_on: freq={:.1}, mode={:?}", freq, self.voice_mode);
        match self.voice_mode {
            VoiceMode::Monophonic => {
                let was_idle = self.monophonic_stage == EnvelopeStage::Idle;
                let should_retrigger = self.retrigger_envelope || was_idle;

                self.monophonic_note = Some(freq);

                if should_retrigger {
                    self.monophonic_phase = 0.0;
                    self.monophonic_osc_phases = [0.0; 4];
                    self.monophonic_envelope = 0.0;
                    self.monophonic_stage = EnvelopeStage::Attack;
                } else {
                    // Без ретриггера — плавно переводим фазу, огибающую не сбрасываем.
                    if self.monophonic_stage == EnvelopeStage::Release {
                        self.monophonic_stage = EnvelopeStage::Attack;
                    }
                    self.smooth_phase_transition(freq);
                }

                self.monophonic_key_held = true;
                self.legato_active = false;
                self.legato_note_stack.clear();
                self.current_frequency = freq;
                self.target_frequency = freq;
            }
            VoiceMode::Legato => {
                self.legato_note_stack.retain(|&f| (f - freq).abs() > 0.1);
                self.legato_note_stack.push(freq);
                let active_freq = *self.legato_note_stack.last().unwrap();
                if self.monophonic_note.is_none() || !self.legato_active {
                    self.monophonic_note = Some(active_freq);
                    self.monophonic_phase = 0.0;
                    self.monophonic_osc_phases = [0.0; 4];
                    self.monophonic_stage = EnvelopeStage::Attack;
                    self.monophonic_envelope = 0.0;
                    self.monophonic_key_held = true;
                    self.legato_active = true;
                    self.current_frequency = active_freq;
                    self.target_frequency = active_freq;
                } else {
                    self.monophonic_note = Some(active_freq);
                    self.current_frequency = active_freq;
                    self.target_frequency = active_freq;
                }
            }
            VoiceMode::Polyphonic => {
                let mut found = false;
                if self.retrigger_envelope {
                    for (i, v) in self.polyphonic_voices.iter_mut().enumerate() {
                        if v.envelope_stage == EnvelopeStage::Release {
                            v.reset();
                            v.note_on(freq, self.sample_rate);
                            found = true;
                            self.voice_stealing_index = (i + 1) % 8;
                            break;
                        }
                    }
                }
                if !found {
                    for (i, v) in self.polyphonic_voices.iter_mut().enumerate() {
                        if !v.is_active() {
                            v.note_on(freq, self.sample_rate);
                            found = true;
                            self.voice_stealing_index = (i + 1) % 8;
                            break;
                        }
                    }
                }
                if !found {
                    let v = &mut self.polyphonic_voices[self.voice_stealing_index];
                    v.reset();
                    v.note_on(freq, self.sample_rate);
                    self.voice_stealing_index = (self.voice_stealing_index + 1) % 8;
                }
            }
            VoiceMode::Portamento => {
                if self.retrigger_envelope && self.monophonic_stage == EnvelopeStage::Release {
                    self.monophonic_stage = EnvelopeStage::Idle;
                    self.monophonic_envelope = 0.0;
                    self.legato_note_stack.clear();
                }
                self.legato_note_stack.retain(|&f| (f - freq).abs() > 0.1);
                self.legato_note_stack.push(freq);
                let active_freq = *self.legato_note_stack.last().unwrap();
                if self.monophonic_note.is_none() || self.monophonic_stage == EnvelopeStage::Idle {
                    self.monophonic_note = Some(active_freq);
                    self.monophonic_phase = 0.0;
                    self.monophonic_osc_phases = [0.0; 4];
                    self.monophonic_stage = EnvelopeStage::Attack;
                    self.monophonic_envelope = 0.0;
                    self.monophonic_key_held = true;
                    self.legato_active = true;
                    self.current_frequency = active_freq;
                    self.target_frequency = active_freq;
                } else {
                    let r: f32 = self.rng.gen();
                    if r > 0.8 {
                        self.monophonic_phase *= 0.9;
                        for p in &mut self.monophonic_osc_phases { *p *= 0.9; }
                    }
                    self.monophonic_note = Some(active_freq);
                    self.target_frequency = active_freq;
                    if self.monophonic_stage == EnvelopeStage::Idle {
                        self.monophonic_stage = EnvelopeStage::Attack;
                        self.monophonic_envelope = 0.0;
                    }
                }
                self.monophonic_key_held = true;
            }
        }
    }

    pub fn note_off(&mut self, freq: f32) {
        log::debug!("note_off: freq={:.1}, mode={:?}", freq, self.voice_mode);
        match self.voice_mode {
            VoiceMode::Monophonic => {
                if let Some(cf) = self.monophonic_note {
                    if (cf - freq).abs() < 0.1 {
                        self.monophonic_key_held = false;
                        self.monophonic_stage = EnvelopeStage::Release;
                        self.legato_active = false;
                    }
                }
            }
            VoiceMode::Legato => {
                self.legato_note_stack.retain(|&f| (f - freq).abs() > 0.1);
                if self.legato_note_stack.is_empty() {
                    self.monophonic_key_held = false;
                    self.monophonic_stage = EnvelopeStage::Release;
                    self.legato_active = false;
                } else {
                    let active = *self.legato_note_stack.last().unwrap();
                    self.monophonic_note = Some(active);
                    self.current_frequency = active;
                    self.target_frequency = active;
                }
            }
            VoiceMode::Polyphonic => {
                for v in &mut self.polyphonic_voices {
                    if v.frequency > 0.0
                        && (v.frequency - freq).abs() < 0.1
                        && v.is_active()
                    {
                        v.note_off();
                    }
                }
            }
            VoiceMode::Portamento => {
                self.legato_note_stack.retain(|&f| (f - freq).abs() > 0.1);
                if self.legato_note_stack.is_empty() {
                    self.monophonic_key_held = false;
                    self.monophonic_stage = EnvelopeStage::Release;
                    self.legato_active = false;
                } else {
                    let active = *self.legato_note_stack.last().unwrap();
                    self.monophonic_note = Some(active);
                    self.target_frequency = active;
                }
            }
        }
    }

    pub fn active_notes(&self) -> Vec<f32> {
        match self.voice_mode {
            VoiceMode::Monophonic | VoiceMode::Legato | VoiceMode::Portamento => {
                self.monophonic_note.map(|f| vec![f]).unwrap_or_default()
            }
            VoiceMode::Polyphonic => self.polyphonic_voices
                .iter()
                .filter(|v| v.is_active() && v.frequency > 0.0)
                .map(|v| v.frequency)
                .collect(),
        }
    }

    // === Главный процессинг ===
    pub fn process(&mut self, dt: f32) -> (f32, f32) {
        self.time += dt;

        let raw_osc = match self.voice_mode {
            VoiceMode::Monophonic | VoiceMode::Legato | VoiceMode::Portamento => {
                self.process_monophonic_oscillator(dt)
            }
            VoiceMode::Polyphonic => self.process_polyphonic_oscillator(dt),
        };

        let env_mod = self.monophonic_envelope * self.filter_env_amount;
        let keyboard_mod = self.monophonic_note
            .map(|f| (f / self.master_tuning).log2() * self.filter_keyboard_track)
            .unwrap_or(0.0);
        let filtered = filter::process_filter(
            raw_osc,
            self.filter_cutoff,
            self.filter_resonance,
            env_mod,
            keyboard_mod,
            self.power_noise,
            dt,
            &mut self.filter_state,
        );

        let with_artifacts = add_artifacts(
            filtered,
            self.time,
            self.temperature,
            self.power_noise,
            self.component_aging,
            self.voice_mode,
            &mut self.rng,
        );

        let envelope = match self.voice_mode {
            VoiceMode::Monophonic | VoiceMode::Legato | VoiceMode::Portamento => {
                self.process_monophonic_envelope(dt)
            }
            VoiceMode::Polyphonic => 1.0,
        };

        let mut sample = with_artifacts * envelope;

        let sag = self.voltage_sag.update(dt, self.power_noise, self.voice_mode, &mut self.rng);
        sample *= sag;

        let (left, right) = if self.reverb_enabled {
            self.reverb.process_stereo(sample, dt) // <--- ИСПРАВЛЕНО
        } else {
            (sample, sample)
        };

        (left * self.volume, right * self.volume)
    }

    /// Рендерит блок в два моно-буфера (для модульного режима).
    /// Масштаб и клиппинг — те же, что в cpal-колбэке (`audio/mod.rs`).
    pub fn render_block_stereo(&mut self, left: &mut [f32], right: &mut [f32]) {
        let n = left.len().min(right.len());
        let dt = 1.0 / self.sample_rate;
        for i in 0..n {
            let (l, r) = self.process(dt);
            left[i] = (l * 0.3).clamp(-1.0, 1.0);
            right[i] = (r * 0.3).clamp(-1.0, 1.0);
        }
    }

    fn smooth_phase_transition(&mut self, target_freq: f32) {
        let virtual_phase =
            ((self.time as f64 * target_freq as f64).rem_euclid(1.0)) as f32;
        let diff = virtual_phase - self.monophonic_phase;
        let shortest_diff = if diff.abs() > 0.5 { diff - diff.signum() } else { diff };
        self.monophonic_phase =
            (self.monophonic_phase + shortest_diff * 0.2).rem_euclid(1.0);
    }

    fn process_monophonic_oscillator(&mut self, _dt: f32) -> f32 {
        if self.monophonic_note.is_none() { return 0.0; }

        if self.voice_mode == VoiceMode::Portamento {
            let diff = self.target_frequency - self.current_frequency;
            if diff.abs() > 0.1 {
                let speed_factor = 1.0 / (self.portamento_speed.max(0.001) * self.sample_rate);
                self.current_frequency += diff * speed_factor;
            } else {
                self.current_frequency = self.target_frequency;
            }
        } else {
            self.current_frequency = self.target_frequency;
        }

        let base_freq = self.current_frequency;
        let temp_drift = 1.0 + (self.temperature - 25.0) * 0.0005;

        self.monophonic_phase += (base_freq / self.sample_rate) * temp_drift;
        if self.monophonic_phase >= 1.0 { self.monophonic_phase -= 1.0; }

        let mut mix = 0.0;
        for i in 0..4 {
            if self.osc_levels[i] < 0.001 { continue; }
            let octave_mult = 2f32.powi(self.osc_octaves[i]);
            let detune_mult = 2f32.powf(self.osc_detunes[i] / 12.0);
            let osc_freq = base_freq * octave_mult * detune_mult;
            self.monophonic_osc_phases[i] += (osc_freq / self.sample_rate) * temp_drift;
            if self.monophonic_osc_phases[i] >= 1.0 { self.monophonic_osc_phases[i] -= 1.0; }
            mix += generate_waveform(self.osc_wave_shapes[i], self.monophonic_osc_phases[i])
                * self.osc_levels[i];
        }
        mix
    }

    fn process_polyphonic_oscillator(&mut self, dt: f32) -> f32 {
        let mut mix = 0.0;
        let mut active_count = 0;

        for voice in &mut self.polyphonic_voices {
            if voice.envelope_stage == EnvelopeStage::Idle { continue; }
            voice.note_age += dt;

            voice.envelope_value = process_envelope(
                voice.envelope_stage,
                voice.envelope_value,
                voice.is_key_held(),
                self.envelope_attack,
                self.envelope_decay,
                self.envelope_sustain,
                self.envelope_release,
                self.sample_rate,
                dt,
            );
            voice.envelope_stage = update_stage(
                voice.envelope_stage,
                voice.envelope_value,
                voice.is_key_held(),
                self.envelope_sustain,
            );

            if voice.envelope_stage == EnvelopeStage::Idle {
                voice.reset();
                continue;
            }

            let temp_drift = 1.0 + (self.temperature - 25.0) * 0.0005;
            let mut voice_mix = 0.0;
            for i in 0..4 {
                if self.osc_levels[i] < 0.001 { continue; }
                let octave_mult = 2f32.powi(self.osc_octaves[i]);
                let detune_mult = 2f32.powf(self.osc_detunes[i] / 12.0);
                let osc_freq = voice.frequency * octave_mult * detune_mult;
                voice.osc_phases[i] += (osc_freq / self.sample_rate) * temp_drift;
                if voice.osc_phases[i] >= 1.0 { voice.osc_phases[i] -= 1.0; }
                voice_mix += generate_waveform(self.osc_wave_shapes[i], voice.osc_phases[i])
                    * self.osc_levels[i];
            }
            mix += voice_mix * voice.envelope_value;
            active_count += 1;
        }

        if active_count > 0 { mix / (active_count as f32).sqrt() } else { 0.0 }
    }

    fn process_monophonic_envelope(&mut self, dt: f32) -> f32 {
        self.monophonic_envelope = process_envelope(
            self.monophonic_stage,
            self.monophonic_envelope,
            self.monophonic_key_held,
            self.envelope_attack,
            self.envelope_decay,
            self.envelope_sustain,
            self.envelope_release,
            self.sample_rate,
            dt,
        );
        self.monophonic_stage = update_stage(
            self.monophonic_stage,
            self.monophonic_envelope,
            self.monophonic_key_held,
            self.envelope_sustain,
        );

        if self.monophonic_envelope <= 0.001 && self.monophonic_stage == EnvelopeStage::Idle {
            self.monophonic_note = None;
            self.legato_note_stack.clear();
            self.current_frequency = 0.0;
            self.target_frequency = 0.0;
            self.monophonic_phase = 0.0;
            self.monophonic_osc_phases = [0.0; 4];
        }
        self.monophonic_envelope
    }
}

use rand::Rng;