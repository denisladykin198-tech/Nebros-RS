// src/ui/widgets.rs
//! Кастомные виджеты: превью волн, огибающей, АЧХ, клавиатура

use egui::{Color32, FontId, Pos2, Rect, RichText, Sense, Shape, Stroke, Ui, Vec2};
use crate::synth::WaveShape;
use super::theme;

// ───────────────────────── заголовок секции ─────────────────────────

pub fn section_header(ui: &mut Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(text).strong().size(14.0).color(theme::ACCENT_GOLD));
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.5), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, theme::BORDER_LIGHT);
    ui.add_space(2.0);
}

// ───────────────────────── строка слайдера ─────────────────────────

pub fn slider_row<T: egui::emath::Numeric + Copy>(
    ui: &mut Ui,
    label: &str,
    value: &mut T,
    range: std::ops::RangeInclusive<T>,
    format: impl Fn(T) -> String,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).small().color(theme::TEXT_DIM));
        let slider = egui::Slider::new(value, range).show_value(false);
        let slider = if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f32>() {
            slider.logarithmic(true)
        } else {
            slider
        };
        ui.add(slider);
        ui.label(RichText::new(format(*value)).small().monospace().color(theme::TEXT));
    });
}

// ───────────────────────── превью формы волны ─────────────────────────

pub fn waveform_preview(ui: &mut Ui, shape: WaveShape, level: f32) {
    let w = ui.available_width().max(60.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 44.0), Sense::hover());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, egui::Rounding::same(4.0), theme::BG_INSET);
    painter.rect_stroke(rect, egui::Rounding::same(4.0), Stroke::new(1.0, theme::BORDER));

    let mid_y = rect.center().y;
    painter.line_segment(
        [Pos2::new(rect.left() + 4.0, mid_y), Pos2::new(rect.right() - 4.0, mid_y)],
        Stroke::new(1.0, theme::TEXT_MUTED),
    );

    if level <= 0.005 {
        painter.text(
            rect.center(), egui::Align2::CENTER_CENTER,
            "—", FontId::proportional(18.0), theme::TEXT_MUTED,
        );
        return;
    }

    let color = match shape {
        WaveShape::Sawtooth => theme::ACCENT_ORANGE,
        WaveShape::Square   => theme::ACCENT_BLUE,
        WaveShape::Triangle => theme::ACCENT_GREEN,
    };

    let amp = rect.height() * 0.42 * level.clamp(0.0, 1.0);
    let steps = 140;
    let cycles = 2.0_f32;
    let mut pts = Vec::with_capacity(steps + 1);

    for i in 0..=steps {
        let t = (i as f32 / steps as f32) * cycles;
        let phase = t.fract();
        let y_norm = match shape {
            WaveShape::Sawtooth => 2.0 * phase - 1.0,
            WaveShape::Square   => if phase < 0.5 { 1.0 } else { -1.0 },
            WaveShape::Triangle => if phase < 0.5 { 4.0 * phase - 1.0 } else { 3.0 - 4.0 * phase },
        };
        let x = rect.left() + 4.0 + (i as f32 / steps as f32) * (rect.width() - 8.0);
        pts.push(Pos2::new(x, mid_y - y_norm * amp));
    }

    painter.add(Shape::line(pts.clone(), Stroke::new(3.5, color.linear_multiply(0.15))));
    painter.add(Shape::line(pts, Stroke::new(1.6, color)));
}

// ───────────────────────── превью огибающей ADSR ─────────────────────────

pub fn envelope_preview(ui: &mut Ui, a: f32, d: f32, s: f32, r: f32) {
    let w = ui.available_width().max(100.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 64.0), Sense::hover());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, egui::Rounding::same(4.0), theme::BG_INSET);
    painter.rect_stroke(rect, egui::Rounding::same(4.0), Stroke::new(1.0, theme::BORDER));

    let inner = rect.shrink(8.0);
    let base = inner.bottom();
    let top = inner.top();

    let aw = (a / (a + d + r + 0.3)).clamp(0.02, 0.6);
    let dw = (d / (a + d + r + 0.3)).clamp(0.02, 0.6);
    let sw = 0.25_f32;
    let rw = (r / (a + d + r + 0.3)).clamp(0.02, 0.6);
    let sum = aw + dw + sw + rw;

    let w = inner.width();
    let x0 = inner.left();
    let x1 = x0 + w * (aw / sum);
    let x2 = x1 + w * (dw / sum);
    let x3 = x2 + w * (sw / sum);
    let x4 = x3 + w * (rw / sum);

    let s_y = base - s * inner.height();
    let pts = vec![
        Pos2::new(x0, base),
        Pos2::new(x1, top),
        Pos2::new(x2, s_y),
        Pos2::new(x3, s_y),
        Pos2::new(x4, base),
    ];

    // Заливка
    let mut fill = pts.clone();
    fill.push(Pos2::new(x4, base));
    fill.push(Pos2::new(x0, base));
    painter.add(Shape::Path(egui::epaint::PathShape {
        points: fill,
        closed: true,
        fill: theme::ACCENT_GOLD.linear_multiply(0.12),
        stroke: Stroke::NONE,
    }));

    // Контур
    painter.add(Shape::line(pts, Stroke::new(2.0, theme::ACCENT_GOLD)));

    // Маркеры
    for p in [Pos2::new(x1, top), Pos2::new(x2, s_y), Pos2::new(x3, s_y)] {
        painter.circle_filled(p, 2.8, theme::ACCENT_GOLD);
    }

    // Подписи
    let font = FontId::proportional(9.0);
    painter.text(Pos2::new(x1, top + 2.0), egui::Align2::CENTER_TOP, "A", font.clone(), theme::TEXT_MUTED);
    painter.text(Pos2::new((x2 + x3) / 2.0, top + 2.0), egui::Align2::CENTER_TOP, "S", font.clone(), theme::TEXT_MUTED);
    painter.text(Pos2::new((x3 + x4) / 2.0, top + 2.0), egui::Align2::CENTER_TOP, "R", font, theme::TEXT_MUTED);
}

// ───────────────────────── АЧХ фильтра ─────────────────────────

pub fn filter_curve(ui: &mut Ui, cutoff: f32, resonance: f32) {
    let w = ui.available_width().max(100.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 64.0), Sense::hover());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, egui::Rounding::same(4.0), theme::BG_INSET);
    painter.rect_stroke(rect, egui::Rounding::same(4.0), Stroke::new(1.0, theme::BORDER));

    let inner = rect.shrink(8.0);
    let f_min = 20f32.ln();
    let f_max = 20000f32.ln();
    let cutoff_ln = cutoff.clamp(20.0, 20000.0).ln();
    let cutoff_x = inner.left() + ((cutoff_ln - f_min) / (f_max - f_min)) * inner.width();

    let steps = 100;
    let mut pts = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let f = (f_min + t * (f_max - f_min)).exp();
        let wr = (f / cutoff).max(1e-3);

        // Q = 1/(2ζ); при res=0 → 0.707 (Баттерворт, без горба),
        // при res→1 растёт до ~4.7 (выраженный пик на срезе).
        let q = 0.707 + resonance * 4.0;
        let denom = ((1.0 - wr * wr).powi(2) + (wr / q).powi(2))
            .sqrt()
            .max(1e-3);
        let mag = (1.0 / denom).min(1.6);

        let y = inner.bottom() - mag * inner.height() * 0.85;
        let x = inner.left() + t * inner.width();
        pts.push(Pos2::new(x, y));
    }

    let mut fill = pts.clone();
    fill.push(Pos2::new(inner.right(), inner.bottom()));
    fill.push(Pos2::new(inner.left(), inner.bottom()));
    painter.add(Shape::Path(egui::epaint::PathShape {
        points: fill,
        closed: true,
        fill: theme::ACCENT_BLUE.linear_multiply(0.12),
        stroke: Stroke::NONE,
    }));
    painter.add(Shape::line(pts, Stroke::new(1.6, theme::ACCENT_BLUE)));

    painter.line_segment(
        [Pos2::new(cutoff_x, inner.top()), Pos2::new(cutoff_x, inner.bottom())],
        Stroke::new(1.0, theme::ACCENT_ORANGE.linear_multiply(0.55)),
    );

    painter.text(
        Pos2::new(inner.left() + 3.0, inner.top() + 1.0),
        egui::Align2::LEFT_TOP,
        format!("{:.0} Hz", cutoff),
        FontId::proportional(9.0),
        theme::TEXT_DIM,
    );
}

// ───────────────────────── LED ─────────────────────────

pub fn led(ui: &mut Ui, active: bool, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(9.0), Sense::hover());
    let c = rect.center();
    if active {
        ui.painter().circle_filled(c, 6.0, color.linear_multiply(0.18));
        ui.painter().circle_filled(c, 3.5, color);
    } else {
        ui.painter().circle_filled(c, 3.5, color.linear_multiply(0.22));
    }
}

// ───────────────────────── экранная клавиатура ─────────────────────────

/// Рисует многооктавную клавиатуру, обрабатывает клики/перетаскивание,
/// возвращает (note_on_midi, note_off_midi).
///
/// * `active_midis`   — MIDI-ноты, реально звучащие сейчас (для подсветки)
/// * `base_midi`      — MIDI-нота крайней левой белой клавиши (например, 36 = C2)
/// * `num_octaves`    — сколько полных октав показать (плюс финальная C сверху)
/// * `computer_octave`— индекс октавы (0..num_octaves-1), на которой показать буквы A..K
/// * `pressed_key`    — MIDI-нота, зажатая мышью; хранится между кадрами
pub fn piano_keyboard(
    ui: &mut Ui,
    active_midis: &[u8],
    base_midi: u8,
    num_octaves: u8,
    computer_octave: u8,
    pressed_key: &mut Option<u8>,
) -> (Option<u8>, Option<u8>) {
    let mut note_on: Option<u8> = None;
    let mut note_off: Option<u8> = None;

    // 7 белых × N октав + финальная C
    let total_whites = 7 * num_octaves as usize + 1;
    let h = 110.0;
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), h),
        Sense::click_and_drag(),
    );
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, egui::Rounding::same(6.0), theme::BG_INSET);

    let kw = rect.width() / total_whites as f32;

    // Семитоны белых клавиш от C: C D E F G A B
    let white_semitones: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];
    // Чёрные: (семитон, индекс белой клавиши слева)
    let black_info: [(i32, usize); 5] = [
        (1, 0),   // C# между C и D
        (3, 1),   // D# между D и E
        (6, 3),   // F# между F и G
        (8, 4),   // G# между G и A
        (10, 5),  // A# между A и B
    ];
    let white_letters = ["A", "S", "D", "F", "G", "H", "J"];
    let black_letters = ["W", "E", "T", "Y", "U"];

    // ── Белые клавиши ──
    let mut white_rects: Vec<(Rect, u8)> = Vec::with_capacity(total_whites);
    for oct in 0..num_octaves as i32 {
        for (i, &semi) in white_semitones.iter().enumerate() {
            let w_idx = (oct as usize) * 7 + i;
            let x0 = rect.left() + w_idx as f32 * kw;
            let kr = Rect::from_min_size(
                Pos2::new(x0 + 1.0, rect.top() + 2.0),
                Vec2::new(kw - 2.0, rect.height() - 4.0),
            );
            let midi = (base_midi as i32 + oct * 12 + semi) as u8;
            white_rects.push((kr, midi));

            let is_active = active_midis.contains(&midi);
            let fill = if is_active { theme::ACCENT } else { Color32::from_rgb(240, 240, 245) };
            painter.rect_filled(kr, egui::Rounding::same(3.0), fill);
            painter.rect_stroke(kr, egui::Rounding::same(3.0), Stroke::new(1.0, theme::BORDER));

            // Подпись «C2 / C3 / …» на каждой C
            if semi == 0 {
                let fg = if is_active { Color32::WHITE } else { Color32::from_rgb(60, 60, 80) };
                painter.text(
                    Pos2::new(kr.center().x, kr.bottom() - 6.0),
                    egui::Align2::CENTER_BOTTOM,
                    format!("C{}", (midi as i32 / 12) - 1),
                    FontId::proportional(10.0),
                    fg,
                );
            }
        }
    }
    // Финальная C справа
    {
        let w_idx = num_octaves as usize * 7;
        let x0 = rect.left() + w_idx as f32 * kw;
        let kr = Rect::from_min_size(
            Pos2::new(x0 + 1.0, rect.top() + 2.0),
            Vec2::new(kw - 2.0, rect.height() - 4.0),
        );
        let midi = base_midi + num_octaves * 12;
        white_rects.push((kr, midi));
        let is_active = active_midis.contains(&midi);
        let fill = if is_active { theme::ACCENT } else { Color32::from_rgb(240, 240, 245) };
        painter.rect_filled(kr, egui::Rounding::same(3.0), fill);
        painter.rect_stroke(kr, egui::Rounding::same(3.0), Stroke::new(1.0, theme::BORDER));
        let fg = if is_active { Color32::WHITE } else { Color32::from_rgb(60, 60, 80) };
        painter.text(
            Pos2::new(kr.center().x, kr.bottom() - 6.0),
            egui::Align2::CENTER_BOTTOM,
            format!("C{}", (midi as i32 / 12) - 1),
            FontId::proportional(10.0),
            fg,
        );
    }

    // ── Чёрные клавиши ──
    let mut black_rects: Vec<(Rect, u8)> = Vec::with_capacity(5 * num_octaves as usize);
    for oct in 0..num_octaves as i32 {
        for &(semi, after) in black_info.iter() {
            // Позиция: сразу за `after`-й белой клавишей этой октавы
            let cx = rect.left() + (oct as f32 * 7.0 + after as f32 + 1.0) * kw;
            let bw = kw * 0.6;
            let bh = rect.height() * 0.62;
            let kr = Rect::from_min_size(
                Pos2::new(cx - bw / 2.0, rect.top() + 2.0),
                Vec2::new(bw, bh),
            );
            let midi = (base_midi as i32 + oct * 12 + semi) as u8;
            black_rects.push((kr, midi));

            let is_active = active_midis.contains(&midi);
            let fill = if is_active { theme::ACCENT } else { Color32::from_rgb(22, 23, 28) };
            painter.rect_filled(kr, egui::Rounding::same(2.0), fill);
            painter.rect_stroke(kr, egui::Rounding::same(2.0), Stroke::new(1.0, Color32::BLACK));
        }
    }

    // ── Буквы A..K только на «компьютерной» октаве ──
    let co = computer_octave as usize;
    for (i, letter) in white_letters.iter().enumerate() {
        let w_idx = co * 7 + i;
        if let Some(&(kr, midi)) = white_rects.get(w_idx) {
            let is_active = active_midis.contains(&midi);
            painter.text(
                Pos2::new(kr.center().x, kr.bottom() - 22.0),
                egui::Align2::CENTER_BOTTOM,
                *letter,
                FontId::proportional(11.0),
                if is_active { Color32::WHITE } else { Color32::from_rgb(100, 100, 120) },
            );
        }
    }
    // K = первая C следующей октавы (финальная C на последней октаве)
    let k_idx = co * 7 + 7;
    if let Some(&(kr, midi)) = white_rects.get(k_idx) {
        let is_active = active_midis.contains(&midi);
        painter.text(
            Pos2::new(kr.center().x, kr.bottom() - 22.0),
            egui::Align2::CENTER_BOTTOM,
            "K",
            FontId::proportional(11.0),
            if is_active { Color32::WHITE } else { Color32::from_rgb(100, 100, 120) },
        );
    }
    for (i, letter) in black_letters.iter().enumerate() {
        let b_idx = co * 5 + i; // в каждой октаве 5 чёрных, порядок совпадает с black_info
        if let Some(&(kr, midi)) = black_rects.get(b_idx) {
            let is_active = active_midis.contains(&midi);
            painter.text(
                Pos2::new(kr.center().x, kr.bottom() - 6.0),
                egui::Align2::CENTER_BOTTOM,
                *letter,
                FontId::proportional(9.0),
                if is_active { Color32::WHITE } else { Color32::from_rgb(190, 195, 210) },
            );
        }
    }

    // ── Взаимодействие мышью ──
    if response.is_pointer_button_down_on() {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            // Чёрные проверяем первыми — они сверху
            let hit = black_rects.iter().find(|(r, _)| r.contains(pos))
                .or_else(|| white_rects.iter().find(|(r, _)| r.contains(pos)))
                .map(|(_, m)| *m);

            if let Some(new_midi) = hit {
                if *pressed_key != Some(new_midi) {
                    if let Some(old) = pressed_key.take() {
                        note_off = Some(old);
                    }
                    *pressed_key = Some(new_midi);
                    note_on = Some(new_midi);
                }
            }
            // Если курсор ушёл за пределы клавиш — текущая нота продолжает звучать
            // (как glissando на реальном синтезаторе), отпустим по mouse-up.
        }
    } else if pressed_key.is_some() {
        if let Some(old) = pressed_key.take() {
            note_off = Some(old);
        }
    }

    (note_on, note_off)
}