mod api;
mod cache;
mod config;
mod game;
mod games;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use config::Settings;
use game::GameProvider;
use games::minecraft::minecraft;
use std::{sync::Arc, time::Duration};
use tracing::{error, info};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        api::v1::routes::list_games,
        api::v1::routes::list_versions,
        api::v1::routes::list_builds,
        api::v1::routes::download_version,
    ),
    components(
        schemas(
            api::v1::models::LoaderInfo,
            api::v1::models::GameInfo,
            api::v1::models::VersionInfo,
            api::v1::models::BuildInfo,
            api::v1::models::ErrorResponse,
            api::v1::models::VersionQuery,
            api::v1::models::BuildQuery,
            api::v1::models::DownloadQuery,
        )
    ),
    tags(
        (name = "warehouse", description = "Pyro warehouse API"),
    )
)]
struct ApiDoc;

#[utoipa::path(
    get,
    path = "/",
    tag = "warehouse",
    responses(
        (status = 200, description = "API root", body = String)
    )
)]
#[get("/")]
async fn api_root() -> impl Responder {
    HttpResponse::Ok().body(format!("running warehouse {}", env!("CARGO_PKG_VERSION")))
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub games: Arc<GameProvider>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::new().expect("Failed to load configuration");

    tracing_subscriber::fmt()
        .with_max_level(
            settings
                .log_level
                .parse::<tracing::Level>()
                .expect("Invalid log level"),
        )
        .init();

    info!("starting warehouse {}", env!("CARGO_PKG_VERSION"));

    let games = Arc::new(GameProvider::from_settings(&settings));
    games.register_game(minecraft()).await;

    let games_clone = games.clone();
    tokio::spawn(async move {
        loop {
            if let Err(e) = games_clone.cache.cleanup().await {
                error!("cache cleanup failed: {}", e);
            }
            tokio::time::sleep(Duration::from_secs(settings.cache_ttl)).await;
        }
    });

    let app_state = AppState { games };

    let bind_address = settings.bind_address.clone();
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(actix_web::middleware::Logger::default())
            .service(SwaggerUi::new("/docs/{_:.*}").url("/docs/openapi.json", ApiDoc::openapi()))
            .service(api_root)
            .configure(api::v1::configure)
    })
    .bind(bind_address)?
    .run();

    tokio::select! {
        res = async {
            info!("starting server at http://{}", settings.bind_address);
            server.await
        } => res?,
        _ = tokio::signal::ctrl_c() => {
            info!("received Ctrl+C, shutting down");
        },
    }

    Ok(())
}
