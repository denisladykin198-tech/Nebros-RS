// src/module_entry.rs
//! Dansel Engine ABI v2 — точка входа модуля-синта Polivoks RS.
//! Компилируется только с `--features module`.

use crate::synth::{PolivoksSynth, VoiceMode, WaveShape};
use std::ffi::c_void;
use std::os::raw::c_char;
use std::sync::atomic::{AtomicPtr, Ordering};

pub const DANSEL_API_VERSION: u32 = 2;

#[repr(C)] #[derive(Clone, Copy)]
pub enum ModuleKind { Synth = 0, Effect = 1, Tool = 2, Game = 3, Unknown = 99 }

#[repr(C)]
pub struct ModuleInfo {
    pub name: *const c_char,
    pub version: u32,
    pub kind: ModuleKind,
    pub api_version: u32,
}

#[repr(C)] #[derive(Clone, Copy)]
pub struct MidiEvent {
    pub sample_offset: u32,
    pub status: u8,
    pub data1: u8,
    pub data2: u8,
}

#[repr(C)]
pub struct ModuleApi {
    pub info: ModuleInfo,
    pub create: unsafe extern "C" fn() -> *mut c_void,
    pub destroy: unsafe extern "C" fn(*mut c_void),
    pub prepare: unsafe extern "C" fn(*mut c_void, f32, u32),
    pub process: unsafe extern "C" fn(
        *mut c_void, *const *const f32, *mut *mut f32, u32, u32,
    ),
    pub process_midi: unsafe extern "C" fn(
        *mut c_void, *const MidiEvent, u32,
        *const *const f32, *mut *mut f32, u32, u32,
    ),
    pub has_editor: unsafe extern "C" fn(*mut c_void) -> bool,
    pub get_editor_size: unsafe extern "C" fn(*mut c_void, *mut u32, *mut u32),
    pub open_editor: unsafe extern "C" fn(*mut c_void, *mut c_void) -> bool,
    pub close_editor: unsafe extern "C" fn(*mut c_void),
    pub editor_idle: unsafe extern "C" fn(*mut c_void),
}

struct Instance {
    synth: PolivoksSynth,
}

const NAME: &[u8] = b"Polivoks RS\0";
static API: AtomicPtr<ModuleApi> = AtomicPtr::new(std::ptr::null_mut());

fn setup_defaults(synth: &mut PolivoksSynth) {
    let waves = [
        WaveShape::Sawtooth,
        WaveShape::Square,
        WaveShape::Triangle,
        WaveShape::Sawtooth,
    ];
    let levels = [1.0, 0.5, 0.3, 0.0];
    for i in 0..4 {
        synth.set_osc_wave_shape(i, waves[i]);
        synth.set_osc_level(i, levels[i]);
        synth.set_osc_detune(i, 0.0);
        synth.set_osc_octave(i, 0);
    }
    synth.set_adsr(0.01, 0.1, 0.7, 0.5);
    synth.set_cutoff(1000.0);
    synth.set_resonance(0.3);
    synth.set_filter_env_amount(0.5);
    synth.set_filter_keyboard_track(0.5);
    synth.set_temperature(25.0);
    synth.set_power_noise(0.1);
    synth.set_component_aging(0.0);
    synth.set_master_tuning(440.0);
    synth.set_voice_mode(VoiceMode::Monophonic);
    synth.set_portamento_speed(0.1);
    synth.set_retrigger_envelope(false);
    synth.set_reverb_enabled(false);
    synth.set_volume(0.7);
}

unsafe extern "C" fn create() -> *mut c_void {
    let mut synth = PolivoksSynth::new();
    setup_defaults(&mut synth);
    Box::into_raw(Box::new(Instance { synth })) as *mut c_void
}

unsafe extern "C" fn destroy(p: *mut c_void) {
    if !p.is_null() {
        drop(Box::from_raw(p as *mut Instance));
    }
}

unsafe extern "C" fn prepare(p: *mut c_void, sr: f32, _mb: u32) {
    if p.is_null() { return; }
    let inst = &mut *(p as *mut Instance);
    // Пересоздаёт реверб под новую частоту, sample_rate сохраняется внутри синта.
    inst.synth.set_sample_rate(sr);
}

unsafe extern "C" fn process(
    _instance: *mut c_void,
    _inputs: *const *const f32,
    _outputs: *mut *mut f32,
    _ch: u32,
    _frames: u32,
) {
    // Синт не обрабатывает аудио-вход; всё через process_midi.
}

unsafe extern "C" fn process_midi(
    instance: *mut c_void,
    events: *const MidiEvent,
    num_events: u32,
    _inputs: *const *const f32,
    outputs: *mut *mut f32,
    _num_channels: u32,
    num_frames: u32,
) {
    if instance.is_null() || outputs.is_null() || num_frames == 0 { return; }
    let inst = &mut *(instance as *mut Instance);
    let synth = &mut inst.synth;

    // 1. MIDI → note_on / note_off
    if !events.is_null() && num_events > 0 {
        let events = std::slice::from_raw_parts(events, num_events as usize);
        for ev in events {
            match ev.status & 0xF0 {
                0x90 if ev.data2 > 0 => {
                    synth.note_on(midi_note_to_freq(ev.data1));
                }
                0x80 | 0x90 => {
                    synth.note_off(midi_note_to_freq(ev.data1));
                }
                _ => {} // CC / PitchBend пока игнорируем
            }
        }
    }

    // 2. Рендер стерео-блока (проверяем оба указателя)
    let left = *outputs.add(0);
    let right = *outputs.add(1);
    if left.is_null() || right.is_null() { return; }
    let out_l = std::slice::from_raw_parts_mut(left, num_frames as usize);
    let out_r = std::slice::from_raw_parts_mut(right, num_frames as usize);
    synth.render_block_stereo(out_l, out_r);
}

fn midi_note_to_freq(note: u8) -> f32 {
    440.0 * 2f32.powf((note as f32 - 69.0) / 12.0)
}

unsafe extern "C" fn has_editor(_: *mut c_void) -> bool { false }
unsafe extern "C" fn get_editor_size(_: *mut c_void, w: *mut u32, h: *mut u32) {
    if !w.is_null() { *w = 0; }
    if !h.is_null() { *h = 0; }
}
unsafe extern "C" fn open_editor(_: *mut c_void, _: *mut c_void) -> bool { false }
unsafe extern "C" fn close_editor(_: *mut c_void) {}
unsafe extern "C" fn editor_idle(_: *mut c_void) {}

#[no_mangle]
pub unsafe extern "C" fn dansel_get_api() -> *const ModuleApi {
    let mut p = API.load(Ordering::Acquire);
    if p.is_null() {
        let api = Box::new(ModuleApi {
            info: ModuleInfo {
                name: NAME.as_ptr() as *const c_char,
                version: 1,
                kind: ModuleKind::Synth,
                api_version: DANSEL_API_VERSION,
            },
            create, destroy, prepare, process, process_midi,
            has_editor, get_editor_size, open_editor, close_editor, editor_idle,
        });
        p = Box::into_raw(api);
        API.store(p, Ordering::Release);
    }
    p
}