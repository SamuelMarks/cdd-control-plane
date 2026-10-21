#![cfg(not(tarpaulin_include))]
#![cfg_attr(coverage, allow(unused_imports))]
//! Main binary entry point for the `cdd-control-plane` HTTP server.

use actix_web::{middleware, web, App, HttpServer};
use cdd_control_plane::api;
use cdd_control_plane::config::AppConfig;
use cdd_control_plane::db::repository::{CddRepository, DbPool, PgRepository};
use cdd_control_plane::queue::{JobQueue, QueueClient};
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use std::sync::Arc;

/// Starts the `cdd-control-plane` HTTP server with PostgreSQL and Redis integrations.
#[actix_web::main]
#[cfg(not(coverage))]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = match AppConfig::load(None) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to load configuration: {e}");
            std::process::exit(1);
        }
    };

    let server_bind = config.server_bind.clone();

    let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
    let db_pool: DbPool = match diesel::r2d2::Pool::builder().build(manager) {
        Ok(p) => p,
        Err(e) => {
            log::error!("Failed to connect to database: {e}");
            std::process::exit(1);
        }
    };

    let queue_client = match QueueClient::new(&config).await {
        Ok(q) => q,
        Err(e) => {
            log::error!("Failed to connect to Redis queue: {e}");
            std::process::exit(1);
        }
    };

    let repo: Arc<dyn CddRepository> = Arc::new(PgRepository { pool: db_pool });
    let queue: Arc<dyn JobQueue> = Arc::new(queue_client);

    let config_data = web::Data::new(config);
    let repo_data = web::Data::new(repo);
    let queue_data = web::Data::new(queue);

    log::info!("Starting cdd-control-plane server on {server_bind}");

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(config_data.clone())
            .app_data(repo_data.clone())
            .app_data(queue_data.clone())
            .configure(api::configure)
    })
    .bind(&server_bind)?
    .run()
    .await
}
