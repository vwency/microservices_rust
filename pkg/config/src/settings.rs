use crate::config::{AppConfig, RawConfig};
use std::error::Error;

pub fn load_config(service_name: &str) -> Result<AppConfig, Box<dyn Error>> {
    let config_path = format!("configs/{}/config", service_name);

    // Создаем настройки, загружая конфигурацию из файла
    let settings = ::config::Config::builder()
        .add_source(::config::File::with_name(&config_path).required(true))
        .build()?;

    // Преобразуем файл в структуру RawConfig
    let raw_config: RawConfig = settings.try_deserialize()?;

    // Здесь нам уже не нужно использовать parse для SocketAddr, так как это сделано через кастомный десериализатор
    let address = raw_config.server.address; // Это уже SocketAddr

    // Получаем пути к сертификатам (если есть)
    let tls_cert_path = raw_config.tls.as_ref().and_then(|t| t.cert_path.clone());
    let tls_key_path = raw_config.tls.as_ref().and_then(|t| t.key_path.clone());

    // Возвращаем финализированную структуру конфигурации
    Ok(AppConfig {
        address,
        service_name: raw_config.server.name,
        log_level: raw_config.server.log_level,
        tls_cert_path,
        tls_key_path,
    })
}
