mod application_error;
mod config_error;
mod rgb_error;
mod web_server_error;

pub use application_error::ApplicationError;
pub use config_error::ConfigError;
pub use rgb_error::RGBError;
pub use web_server_error::WebServerError;
