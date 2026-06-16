#![cfg(not(tarpaulin_include))]
//! Main entry point for the cdd-control-plane web server.

use actix_web::{middleware, App, HttpServer};
use cdd_control_plane::api;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("Starting cdd-control-plane server on 0.0.0.0:8080");

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .configure(api::configure)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
