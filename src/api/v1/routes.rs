use crate::api::v1::models::*;
use crate::{game::Version, AppState};
use actix_web::{get, web, HttpResponse, Responder};


#[utoipa::path(
    get,
    path = "/api/v1/versions",  
    tag = "warehouse",
    params(
        ("game" = String, Query, description = "Game identifier"),
        ("loader" = String, Query, description = "Loader identifier"),
        ("stable_only" = bool, Query, description = "Only show stable versions")
    ),
    responses(
        (status = 200, description = "List of versions", body = Vec<VersionInfo>),
        (status = 400, description = "Error response", body = ErrorResponse)
    )
)]
#[get("/versions")]
async fn list_versions(
    state: web::Data<AppState>,
    query: web::Query<VersionQuery>,
) -> impl Responder {
    let loader = match state.games.get_loader(&query.game, &query.loader).await {
        Some(loader) => loader,
        None => {
            return ApiResponse::<Vec<VersionInfo>>::error_response(format!(
                "Loader '{}' not found for game '{}'",
                query.loader, query.game
            ))
        }
    };

    match loader.fetch_versions().await {
        Ok(mut versions) => {
            if query.stable_only {
                versions.retain(|v| v.is_stable());
            }
            let versions = versions
                .into_iter()
                .map(|v| VersionInfo {
                    id: v.id().to_string(),
                    version_type: v.version_type().to_string(),
                    is_stable: v.is_stable(),
                })
                .collect::<Vec<_>>();
            HttpResponse::Ok().json(ApiResponse::success(versions))
        }
        Err(e) => ApiResponse::<Vec<VersionInfo>>::error_response(format!(
            "Failed to fetch versions: {}",
            e
        )),
    }
}


#[utoipa::path(
    get,
    path = "/api/v1/builds",  
    tag = "warehouse",
    params(
        ("game" = String, Query, description = "Game identifier"),
        ("loader" = String, Query, description = "Loader identifier"),
        ("version" = String, Query, description = "Version identifier")
    ),
    responses(
        (status = 200, description = "List of builds", body = Vec<BuildInfo>),
        (status = 400, description = "Error response", body = ErrorResponse)
    )
)]
#[get("/builds")]
async fn list_builds(state: web::Data<AppState>, query: web::Query<BuildQuery>) -> impl Responder {
    let loader = match state.games.get_loader(&query.game, &query.loader).await {
        Some(loader) => loader,
        None => {
            return ApiResponse::<Vec<BuildInfo>>::error_response(format!(
                "Loader '{}' not found for game '{}'",
                query.loader, query.game
            ))
        }
    };

    let version = Version::new_standard(query.version.clone(), "release".to_string());

    match loader.fetch_builds(&version).await {
        Ok(builds) => {
            let builds = builds
                .into_iter()
                .map(|b| BuildInfo {
                    id: b.id().to_string(),
                    version: VersionInfo {
                        id: b.version().id().to_string(),
                        version_type: b.version().version_type().to_string(),
                        is_stable: b.version().is_stable(),
                    },
                    download_url: b.download_url().map(String::from),
                })
                .collect::<Vec<_>>();
            HttpResponse::Ok().json(ApiResponse::success(builds))
        }
        Err(e) => {
            ApiResponse::<Vec<BuildInfo>>::error_response(format!("Failed to fetch builds: {}", e))
        }
    }
}


#[utoipa::path(
    get,
    path = "/api/v1/download",  
    tag = "warehouse",
    params(
        ("game" = String, Query, description = "Game identifier"),
        ("loader" = String, Query, description = "Loader identifier"),
        ("version" = String, Query, description = "Version identifier"),
        ("build_id" = Option<String>, Query, description = "Build identifier")
    ),
    responses(
        (status = 200, description = "Game server JAR file"),
        (status = 400, description = "Error response", body = ErrorResponse)
    )
)]
#[get("/download")]
async fn download_version(
    state: web::Data<AppState>,
    query: web::Query<DownloadQuery>,
) -> impl Responder {
    let loader = match state.games.get_loader(&query.game, &query.loader).await {
        Some(loader) => loader,
        None => {
            return ApiResponse::<Vec<u8>>::error_response(format!(
                "Loader '{}' not found for game '{}'",
                query.loader, query.game
            ))
        }
    };

    let version = Version::new_standard(query.version.clone(), "release".to_string());

    let builds = match loader.fetch_builds(&version).await {
        Ok(builds) => builds,
        Err(e) => {
            return ApiResponse::<Vec<u8>>::error_response(format!("Failed to fetch builds: {}", e))
        }
    };

    let build = if let Some(build_id) = &query.build_id {
        match builds.into_iter().find(|b| b.id() == build_id) {
            Some(b) => b,
            None => {
                return ApiResponse::<Vec<u8>>::error_response(format!(
                    "Build '{}' not found",
                    build_id
                ))
            }
        }
    } else {
        match builds.into_iter().next() {
            Some(b) => b,
            None => {
                return ApiResponse::<Vec<u8>>::error_response(
                    "No builds available for this version".to_string(),
                )
            }
        }
    };

    match state.games.download_build(&query.game, &build).await {
        Ok(data) => HttpResponse::Ok()
            .append_header(("Content-Type", "application/java-archive"))
            .append_header((
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", build.filename()),
            ))
            .body(data),
        Err(e) => {
            ApiResponse::<Vec<u8>>::error_response(format!("Failed to download build: {}", e))
        }
    }
}


#[utoipa::path(
    get,
    path = "/api/v1/games",  
    tag = "warehouse",
    responses(
        (status = 200, description = "List of games", body = Vec<GameInfo>),
        (status = 400, description = "Error response", body = ErrorResponse)
    )
)]
#[get("/games")]
async fn list_games(state: web::Data<AppState>) -> impl Responder {
    let games = state.games.list_games().await;
    let game_infos = games
        .into_iter()
        .map(|game| GameInfo {
            id: game.id().to_string(),
            loaders: game
                .list_loaders()
                .into_iter()
                .map(|loader| LoaderInfo {
                    id: loader.name().to_string(),
                    name: loader.name().to_string(),
                    website: loader.website().map(String::from),
                })
                .collect(),
        })
        .collect::<Vec<_>>();

    HttpResponse::Ok().json(ApiResponse::success(game_infos))
}
