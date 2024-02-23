use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use tracing::{error, warn};

use crate::application::errors::{ApplicationError, RGBError};

const INTERNAL_SERVER_ERROR_MSG: &str =
    "Internal server error, Please contact your administrator or try later";

impl IntoResponse for ApplicationError {
    fn into_response(self) -> Response {
        match self {
            ApplicationError::RGB(error) => error.into_response(),
            _ => {
                error!("{}", self.to_string());

                let status = StatusCode::INTERNAL_SERVER_ERROR;
                let body = generate_body(status, INTERNAL_SERVER_ERROR_MSG);

                (status, body).into_response()
            } // Add additional cases as needed
        }
    }
}

impl IntoResponse for RGBError {
    fn into_response(self) -> Response {
        let (error_message, status) = match self {
            RGBError::Online(_) | RGBError::Invoice(_) => {
                error!("{}", self.to_string());
                (
                    INTERNAL_SERVER_ERROR_MSG.to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            }
            _ => {
                warn!("{}", self.to_string());
                (self.to_string(), StatusCode::BAD_REQUEST)
            }
        };

        let body = generate_body(status, error_message.as_str());
        (status, body).into_response()
    }
}
fn generate_body(status: StatusCode, reason: &str) -> Json<Value> {
    Json(json!({
        "status": status.as_str(),
        "reason": reason,
    }))
}
