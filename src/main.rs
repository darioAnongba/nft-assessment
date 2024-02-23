mod adapters;
mod application;
mod domains;

use std::process::exit;

#[cfg(debug_assertions)]
use dotenv::dotenv;

use crate::adapters::config::config_rs::load_config;
use crate::adapters::logging::tracing::setup_tracing;

use adapters::app::App;
use tracing::error;

#[tokio::main]
async fn main() {
    // Load .env file in development
    #[cfg(debug_assertions)]
    dotenv().ok();

    // Load config and logger
    let config = match load_config() {
        Ok(app_state) => app_state,
        Err(e) => {
            error!(error = ?e);
            exit(1);
        }
    };
    setup_tracing(config.logging.clone());

    let app = match App::new(config.clone()).await {
        Ok(app) => app,
        Err(e) => {
            error!(error = ?e);
            exit(1);
        }
    };

    if let Err(e) = app.start(&config.web.addr).await {
        error!(error = ?e);
        exit(1);
    }
}
