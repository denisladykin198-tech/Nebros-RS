// src/ui/theme.rs

use egui::{Color32, Stroke, Visuals};

pub const BG_DARK: Color32 = Color32::from_rgb(14, 15, 20);
pub const BG_PANEL: Color32 = Color32::from_rgb(22, 24, 32);
pub const BG_CARD: Color32 = Color32::from_rgb(30, 33, 44);
pub const BG_CARD_HOVER: Color32 = Color32::from_rgb(42, 45, 60);
pub const BG_INSET: Color32 = Color32::from_rgb(10, 11, 16);

pub const ACCENT: Color32 = Color32::from_rgb(215, 55, 60);       
pub const ACCENT_GOLD: Color32 = Color32::from_rgb(230, 180, 80); 
pub const ACCENT_GREEN: Color32 = Color32::from_rgb(90, 220, 130);
pub const ACCENT_BLUE: Color32 = Color32::from_rgb(95, 165, 240);
pub const ACCENT_ORANGE: Color32 = Color32::from_rgb(240, 145, 60);
pub const ACCENT_RED: Color32 = Color32::from_rgb(230, 70, 70);

pub const TEXT: Color32 = Color32::from_rgb(228, 230, 238);
pub const TEXT_DIM: Color32 = Color32::from_rgb(150, 155, 175);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(95, 100, 120);

pub const BORDER: Color32 = Color32::from_rgb(52, 57, 72);
pub const BORDER_LIGHT: Color32 = Color32::from_rgb(80, 86, 105);

pub fn apply(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();
    visuals.panel_fill = BG_DARK;
    visuals.window_fill = BG_PANEL;
    visuals.extreme_bg_color = BG_INSET;
    visuals.faint_bg_color = BG_CARD;
    visuals.override_text_color = Some(TEXT);
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.selection.bg_fill = ACCENT;
    visuals.selection.stroke = Stroke::new(1.0, ACCENT_GOLD);

    visuals.widgets.noninteractive.bg_fill = BG_PANEL;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_DIM);

    visuals.widgets.inactive.bg_fill = BG_CARD;
    visuals.widgets.inactive.weak_bg_fill = BG_CARD;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);

    visuals.widgets.hovered.bg_fill = BG_CARD_HOVER;
    visuals.widgets.hovered.weak_bg_fill = BG_CARD_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT_GOLD);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.2, ACCENT_GOLD);

    visuals.widgets.active.bg_fill = ACCENT;
    visuals.widgets.active.weak_bg_fill = ACCENT;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_GOLD);
    visuals.widgets.active.fg_stroke = Stroke::new(1.2, Color32::WHITE);

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    style.spacing.slider_width = 100.0;
    ctx.set_style(style);
}