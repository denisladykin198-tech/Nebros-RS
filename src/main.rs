use nebros_rs::ui::PolivoksApp;
use nebros_rs::consts::APP_TITLE;

fn main() -> Result<(), eframe::Error> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("Запуск Nebros RS v{}", nebros_rs::consts::APP_VERSION);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 1030.0])
            .with_title(APP_TITLE)
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(APP_TITLE, options, Box::new(|cc| Box::new(PolivoksApp::new(cc))))
}