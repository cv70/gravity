use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub struct ApiResponse<T> {
    pub code: u16,
    pub message: Option<String>,
    pub data: Option<T>,
}

impl<T: serde::Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "code": self.code,
            "message": self.message,
            "data": self.data,
        }));

        (
            StatusCode::from_u16(self.code).unwrap_or(StatusCode::OK),
            body,
        )
            .into_response()
    }
}

impl<T: serde::Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 200,
            message: Some("OK".to_string()),
            data: Some(data),
        }
    }

    pub fn created(data: T) -> Self {
        Self {
            code: 201,
            message: Some("Created".to_string()),
            data: Some(data),
        }
    }

    pub fn error(code: u16, message: impl Into<String>) -> Self {
        Self {
            code,
            message: Some(message.into()),
            data: None,
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::error(404, message)
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::error(400, message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::error(401, message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::error(403, message)
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::error(500, message)
    }
}

#[derive(serde::Serialize)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(self),
        )
            .into_response()
    }
}

impl ApiError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            code: 400,
            message: message.into(),
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            code: 401,
            message: message.into(),
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            code: 403,
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            code: 404,
            message: message.into(),
        }
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            code: 500,
            message: message.into(),
        }
    }
}
