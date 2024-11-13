pub mod models;
pub mod routes;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(routes::list_games)
            .service(routes::list_versions)
            .service(routes::list_builds)
            .service(routes::download_version)
    );
}
