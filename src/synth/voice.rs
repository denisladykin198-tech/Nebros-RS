// src/synth/voice.rs
//! Голос синтезатора и стадии огибающей

use super::envelope::EnvelopeStage;

pub struct Voice {
    pub frequency: f32,
    pub phase: f32,
    pub phase_delta: f32,
    pub envelope_value: f32,
    pub envelope_stage: EnvelopeStage,
    pub note_age: f32,
    pub key_held: bool,
    pub osc_phases: [f32; 4],
}

impl Voice {
    pub fn new() -> Self {
        Self {
            frequency: 0.0,
            phase: 0.0,
            phase_delta: 0.0,
            envelope_value: 0.0,
            envelope_stage: EnvelopeStage::Idle,
            note_age: 0.0,
            key_held: false,
            osc_phases: [0.0; 4],
        }
    }

    pub fn note_on(&mut self, freq: f32, sample_rate: f32) {
        self.frequency = freq;
        self.phase = 0.0;
        self.phase_delta = freq / sample_rate;
        self.osc_phases = [0.0; 4];
        self.envelope_stage = EnvelopeStage::Attack;
        self.envelope_value = 0.0;
        self.note_age = 0.0;
        self.key_held = true;
    }

    pub fn note_off(&mut self) {
        self.key_held = false;
        if self.envelope_stage != EnvelopeStage::Idle {
            self.envelope_stage = EnvelopeStage::Release;
        }
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.envelope_stage != EnvelopeStage::Idle
    }

    #[inline]
    pub fn is_key_held(&self) -> bool {
        self.key_held
    }

    pub fn reset(&mut self) {
        self.frequency = 0.0;
        self.phase = 0.0;
        self.phase_delta = 0.0;
        self.osc_phases = [0.0; 4];
        self.envelope_value = 0.0;
        self.envelope_stage = EnvelopeStage::Idle;
        self.note_age = 0.0;
        self.key_held = false;
    }
}