use std::sync::Arc;

use axum::Router;

use tokio::{
    net::TcpListener,
    signal::{
        ctrl_c,
        unix::{signal, SignalKind},
    },
};
use tracing::{debug, error, info, trace};

use crate::{
    adapters::rgb::rgblib::RGBLibClient,
    application::{
        dtos::AppConfig,
        errors::{ApplicationError, WebServerError},
    },
    domains::rgb::api::http::RGBHandler,
};

#[derive(Debug)]
pub struct App {
    router: Router,
}

impl App {
    pub async fn new(config: AppConfig) -> Result<Self, ApplicationError> {
        // Create adapters
        let rgb_client = RGBLibClient::new(config.rgb.clone()).await?;

        // Create services (use cases)
        // let rgb = RGBService::new(Box::new(rgb_client));

        trace!("Initializing app");

        let router = Router::new()
            .nest("/rgb", RGBHandler::routes())
            .with_state(Arc::new(rgb_client));

        debug!("App initialised successfully");
        Ok(Self { router })
    }

    pub async fn start(&self, addr: &str) -> Result<(), WebServerError> {
        trace!("Starting app");

        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| WebServerError::Listener(e.to_string()))?;

        info!(addr, "Listening on");

        axum::serve(listener, self.router.clone())
            .with_graceful_shutdown(Self::shutdown_signal())
            .await
            .map_err(|e| WebServerError::Serve(e.to_string()))?;

        Ok(())
    }

    async fn shutdown_signal() {
        let ctrl_c = async {
            if let Err(e) = ctrl_c().await {
                error!(error = ?e, "Failed to install Ctrl+C handler");
            }
            info!("Received Ctrl+C signal. Shutting down gracefully");
        };

        #[cfg(unix)]
        let terminate = async {
            match signal(SignalKind::terminate()) {
                Ok(mut stream) => {
                    stream.recv().await;
                    info!("Received SIGTERM. Shutting down gracefully");
                }
                Err(e) => error!(error = ?e, "Failed to install SIGTERM handler"),
            }
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }
    }
}
