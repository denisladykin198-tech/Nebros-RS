// src/utils.rs
//! Утилиты: преобразование частоты в ноту, определение аккорда

const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// Преобразует частоту в название ноты (например, 440.0 → "A4")
pub fn freq_to_note_name(freq: f32) -> String {
    let a4_freq = 440.0;
    let semitones = ((freq / a4_freq).log2() * 12.0).round() as i32;
    // A4 = MIDI 69. Отсчёт нот ведём от C (0=C, 9=A).
    let midi_note = 69 + semitones;
    let note_index = midi_note.rem_euclid(12) as usize;
    let octave = midi_note.div_euclid(12) - 1;
    format!("{}{}", NOTE_NAMES[note_index], octave)
}

/// Определяет название аккорда по набору частот
pub fn get_chord_name(notes: &[f32]) -> String {
    if notes.len() < 2 {
        return "Моно".to_string();
    }
    let mut sorted: Vec<f32> = notes.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let intervals: Vec<i32> = sorted
        .windows(2)
        .map(|pair| ((pair[1] / pair[0]).log2() * 12.0).round() as i32)
        .collect();

    match intervals.as_slice() {
        [4, 3] => "Мажор",
        [3, 4] => "Минор",
        [4, 4] => "Увеличенный",
        [3, 3] => "Уменьшенный",
        [7] => "Квинта",
        _ => return format!("Аккорд ({} нот)", notes.len()),
    }
    .to_string()
}

/// Форматирует время: <0.1с → миллисекунды, иначе секунды
pub fn format_time(seconds: f32) -> String {
    if seconds < 0.1 {
        format!("{:.0}мс", seconds * 1000.0)
    } else {
        format!("{:.1}с", seconds)
    }
}