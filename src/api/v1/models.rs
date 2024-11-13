use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, ToSchema)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct LoaderInfo {
    pub id: String,
    pub name: String,
    pub website: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct VersionInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub is_stable: bool,
}

#[derive(Serialize, ToSchema)]
pub struct BuildInfo {
    pub id: String,
    pub version: VersionInfo,
    pub download_url: Option<String>,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct VersionQuery {
    pub game: String,
    pub loader: String,
    #[serde(default)]
    pub stable_only: bool,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct DownloadQuery {
    pub game: String,
    pub loader: String,
    pub version: String,
    pub build_id: Option<String>,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct BuildQuery {
    pub game: String,
    pub loader: String,
    pub version: String,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Serialize, ToSchema)]
pub struct GameInfo {
    pub id: String,
    pub loaders: Vec<LoaderInfo>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: impl Into<String>) -> ApiResponse<T> {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }

    pub fn error_response(msg: impl Into<String>) -> HttpResponse {
        HttpResponse::NotFound().json(Self::error(msg))
    }
}