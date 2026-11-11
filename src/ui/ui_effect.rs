// src/ui/ui_effect.rs
//! UI-структура эффекта

#[derive(Clone, Debug, PartialEq)]
pub enum EffectType {
    Reverb,
    Distortion,
    Chorus,
    Delay,
    Generic, // Для эффектов, чья детальная UI-логика пока не реализована
}

#[derive(Clone, Debug)]
pub struct UiEffect {
    pub name: String,
    pub effect_type: EffectType,
    pub enabled: bool,
    pub parameters: Vec<f32>,
    pub open: bool,
}

impl UiEffect {
    pub fn new(name: &str, effect_type: EffectType) -> Self {
        let param_count = match effect_type {
            EffectType::Reverb => 13,
            _ => 4, // Все остальные эффекты пока имеют по 4 универсальных параметра
        };
        Self {
            name: name.to_string(),
            effect_type,
            enabled: false,
            parameters: vec![0.5; param_count],
            open: false,
        }
    }
}