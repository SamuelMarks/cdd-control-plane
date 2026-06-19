#![warn(clippy::missing_docs_in_private_items)]
#![warn(missing_docs)]
#![cfg(not(tarpaulin_include))]
//! Main entry point for the cdd-control-plane web server.

use actix_web::{middleware, web, App, HttpServer};
use cdd_control_plane::api;
use cdd_control_plane::config::AppConfig;
use cdd_control_plane::db::repository::{CddRepository, DbPool, PgRepository};
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use std::sync::Arc;

/// The main application entry point.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = AppConfig::load(None).unwrap_or_else(|e| {
        log::error!("Failed to load configuration: {}", e);
        std::process::exit(1);
    });

    let server_bind = config.server_bind.clone();

    let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
    let db_pool: DbPool = diesel::r2d2::Pool::builder()
        .build(manager)
        .unwrap_or_else(|e| {
            log::error!("Failed to connect to database: {}", e);
            std::process::exit(1);
        });

    let repo: Arc<dyn CddRepository> = Arc::new(PgRepository {
        pool: db_pool.clone(),
    });

    let config_data = web::Data::new(config.clone());
    let repo_data = web::Data::new(repo);

    log::info!("Starting cdd-control-plane server on {}", server_bind);

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(config_data.clone())
            .app_data(repo_data.clone())
            .configure(api::configure)
    })
    .bind(&server_bind)?
    .run()
    .await
}
