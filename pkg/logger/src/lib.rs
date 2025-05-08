use env_logger::Builder;
use log::{info, LevelFilter};
use std::io::Write;

pub fn init_logger(log_level: &str) {
    info!("Инициализация логгера с уровнем {}", log_level);

    let level_filter = match log_level.to_lowercase().as_str() {
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    };

    Builder::new()
        .filter(None, level_filter)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {}:{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                record.args()
            )
        })
        .init();
}
