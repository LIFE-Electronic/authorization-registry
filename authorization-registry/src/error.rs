use axum::extract::rejection::FormRejection;
use axum::extract::rejection::JsonRejection;
use axum::extract::rejection::PathRejection;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error)]
pub struct ExpectedError {
    pub status_code: StatusCode,
    // message to display to the client
    pub message: String,
    // this is for debug purposes
    pub reason: String,
    // metadata to send to the client
    pub metadata: Option<serde_json::Value>,
}

impl std::fmt::Display for ExpectedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::fmt::Debug for ExpectedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.reason)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Expected(ExpectedError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
    #[error(transparent)]
    JsonExtractorRejection(#[from] JsonRejection),
    #[error(transparent)]
    PathExtractorRejection(#[from] PathRejection),
    #[error(transparent)]
    FormExtratorRejection(#[from] FormRejection),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::FormExtratorRejection(form_rejection) => {
                let message = form_rejection.body_text();
                tracing::error!("Error extracting path from request: '{}'", message);

                let response = ErrorResponse {
                    error: message,
                    metadata: None,
                };

                (StatusCode::BAD_REQUEST, Json(response)).into_response()
            }

            AppError::PathExtractorRejection(path_rejection) => {
                let message = path_rejection.body_text();
                tracing::error!("Error extracting path from request: '{}'", message);

                let response = ErrorResponse {
                    error: message,
                    metadata: None,
                };

                (StatusCode::BAD_REQUEST, Json(response)).into_response()
            }
            AppError::JsonExtractorRejection(json_rejection) => {
                let message = json_rejection.body_text();
                tracing::error!("Error extracting json from request: '{}'", message);

                let response = ErrorResponse {
                    error: message,
                    metadata: None,
                };

                (StatusCode::BAD_REQUEST, Json(response)).into_response()
            }
            AppError::Expected(error) => {
                tracing::info!("{:?}", error);
                let response = ErrorResponse {
                    error: format!("{}", error),
                    metadata: error.metadata,
                };
                return (error.status_code, Json(response)).into_response();
            }
            AppError::Unexpected(error) => {
                tracing::error!("{:?}", error);
                let response = ErrorResponse {
                    error: "Something unexpected went wrong".to_owned(),
                    metadata: None,
                };
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
            }
        }
    }
}

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<serde_json::Value>,
}

impl ErrorResponse {
    pub fn new(error: &str) -> Self {
        ErrorResponse {
            error: error.to_owned(),
            metadata: None,
        }
    }
}
