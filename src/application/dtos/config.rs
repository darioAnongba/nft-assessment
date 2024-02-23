use serde::Deserialize;

use crate::adapters::{
    axum::AxumServerConfig, logging::tracing::TracingLoggerConfig, rgb::rgblib::RGBLibClientConfig,
};

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub rgb: RGBLibClientConfig,
    pub web: AxumServerConfig,
    pub logging: TracingLoggerConfig,
}
