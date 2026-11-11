// src/audio/mod.rs
//! Аудио-подсистема: запуск потока, отслеживание устройств

pub mod device;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::synth::PolivoksSynth;

/// Запускает аудиопоток на устройстве по умолчанию.
/// Возвращает (поток, статус, имя_устройства).
pub fn start_audio(
    synth: Arc<Mutex<PolivoksSynth>>,
    error_flag: Arc<AtomicBool>,
) -> (Option<cpal::Stream>, String, Option<String>) {
    let mut status = "✅ Аудио поток запущен".to_string();
    let mut device_name: Option<String> = None;

    log::info!("🔍 Поиск аудиоустройств...");
    let host = cpal::default_host();

    // Логируем список устройств
    if let Ok(devices) = host.output_devices() {
        let device_list: Vec<String> = devices.filter_map(|d| d.name().ok()).collect();
        log::info!("📋 Доступные аудиоустройства ({}):", device_list.len());
        for (i, name) in device_list.iter().enumerate() {
            log::info!("  {}. {}", i + 1, name);
        }
    }

    let device = match host.default_output_device() {
        Some(device) => {
            match device.name() {
                Ok(name) => {
                    log::info!("🎧 Выбрано устройство: {}", name);
                    device_name = Some(name);
                }
                Err(e) => log::warn!("⚠️ Не удалось получить имя устройства: {}", e),
            }
            device
        }
        None => {
            status = "❌ Не найдено аудиоустройство".to_string();
            log::error!("{}", status);
            return (None, status, None);
        }
    };

    let config = match device.default_output_config() {
        Ok(config) => config,
        Err(e) => {
            status = format!("❌ Ошибка конфигурации: {}", e);
            log::error!("{}", status);
            return (None, status, device_name);
        }
    };

    let sample_rate = config.sample_rate().0 as f32;
    log::info!("🎧 Частота дискретизации: {} Гц", sample_rate);

    synth.lock().set_sample_rate(sample_rate);

    let error_flag_callback = Arc::clone(&error_flag);
    let err_fn = move |err| {
        log::error!("🎹 Аудио ошибка в потоке: {}", err);
        error_flag_callback.store(true, Ordering::SeqCst);
    };

    match config.sample_format() {
        cpal::SampleFormat::F32 => {
            match device.build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut synth = synth.lock();
                    let dt = 1.0 / sample_rate;
                    for frame in data.chunks_mut(2) {
                        let (left, right) = synth.process(dt);
                        // Мягкий клип + нормализация
                        frame[0] = (left * 0.3).clamp(-1.0, 1.0);
                        frame[1] = (right * 0.3).clamp(-1.0, 1.0);
                    }
                },
                err_fn,
                None,
            ) {
                Ok(stream) => match stream.play() {
                    Ok(_) => {
                        log::info!(
                            "✅ Аудио поток запущен на: {:?}",
                            device_name.as_deref().unwrap_or("Неизвестно")
                        );
                        (Some(stream), status, device_name)
                    }
                    Err(e) => {
                        status = format!("❌ Ошибка запуска потока: {}", e);
                        log::error!("{}", status);
                        (None, status, device_name)
                    }
                },
                Err(e) => {
                    status = format!("❌ Ошибка создания потока: {}", e);
                    log::error!("{}", status);
                    (None, status, device_name)
                }
            }
        }
        fmt => {
            status = format!("❌ Неподдерживаемый формат: {:?}", fmt);
            log::error!("{}", status);
            (None, status, device_name)
        }
    }
}

/// Перезапускает аудиопоток (останавливает старый, создаёт новый)
pub fn restart_audio(
    old_stream: &mut Option<cpal::Stream>,
    synth: Arc<Mutex<PolivoksSynth>>,
    error_flag: Arc<AtomicBool>,
    old_device_name: &Option<String>,
) -> (Option<cpal::Stream>, String, Option<String>) {
    log::info!("🔄 Перезапуск аудиопотока...");

    if let Some(stream) = old_stream.take() {
        if let Err(e) = stream.pause() {
            log::warn!("⚠️ Не удалось остановить старый поток: {}", e);
        }
    }

    error_flag.store(false, Ordering::SeqCst);

    let (stream, status, new_device_name) = start_audio(synth, error_flag);

    if let (Some(old), Some(new)) = (old_device_name, &new_device_name) {
        if old != new {
            log::info!("🎯 Устройство изменилось: '{}' -> '{}'", old, new);
        } else {
            log::info!("🔄 Перезапуск на том же устройстве: '{}'", old);
        }
    }

    (stream, status, new_device_name)
}