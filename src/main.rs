use actix_web::{middleware, App, HttpServer};
use dotenv::dotenv;
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgPoolOptions;

use crate::{configs::ApiConfig, configs::DatabaseConfig};

mod app;
mod configs;
mod logging;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    logging::set();

    let api_config = envy::prefixed("API_").from_env::<ApiConfig>().unwrap();

    let db_config = envy::prefixed("DATABASE_")
        .from_env::<DatabaseConfig>()
        .unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(db_config.max_connections)
        .connect(&db_config.url.clone())
        .await
        .expect("Error building a connection pool");

    HttpServer::new(move || {
        App::new()
            .configure(app::configure(pool.clone()))
            .wrap(middleware::Logger::default())
    })
    .bind(api_config.host.to_owned())?
    .run()
    .await
}
