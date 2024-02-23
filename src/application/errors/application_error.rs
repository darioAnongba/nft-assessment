use thiserror::Error;

use super::{ConfigError, RGBError, WebServerError};

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error("RGB Error: {0}")]
    RGB(#[from] RGBError),

    #[error("Web Server Error: {0}")]
    WebServer(#[from] WebServerError),
}
