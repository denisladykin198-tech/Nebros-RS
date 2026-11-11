// src/lib.rs
//! Публичный API.
//! Два режима сборки:
//! - `gui`    — standalone-приложение (main.rs + ui + audio);
//! - `module` — Dansel Engine модуль-синт (module_entry.rs, ABI v2).

pub mod synth;
pub mod effects;
pub mod consts;
pub mod utils;

#[cfg(feature = "module")]
pub mod module_entry;

#[cfg(not(feature = "module"))]
pub mod presets;
#[cfg(not(feature = "module"))]
pub mod audio;
#[cfg(not(feature = "module"))]
pub mod ui;

pub use synth::{PolivoksSynth, WaveShape, VoiceMode};
pub use effects::SovietSpringReverb;