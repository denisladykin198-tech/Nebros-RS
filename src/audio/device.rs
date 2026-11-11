// src/audio/device.rs
//! Отслеживание изменений аудиоустройств

use cpal::traits::{DeviceTrait, HostTrait};

/// Проверяет, изменилось ли аудиоустройство по умолчанию.
/// Возвращает Some(new_name), если устройство изменилось.
pub fn check_current_device(old_name: &Option<String>) -> Option<String> {
    let host = cpal::default_host();
    let device = host.default_output_device()?;
    let current_name = device.name().ok()?;

    match old_name {
        Some(old) if *old != current_name => Some(current_name),
        None => Some(current_name),
        _ => None,
    }
}