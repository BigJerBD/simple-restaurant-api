mod configs;
mod logging;
mod services;

use actix_web::{
    middleware, web, web::ServiceConfig, web::Data, App, HttpServer,
};
use actix_web::web::service;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use sqlx::postgres::PgConnectOptions;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::{configs::DatabaseConfig, configs::ApiConfig};
use crate::services::orders;

pub struct AppContext {
    pub db: Pool<Postgres>
}

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/orders", api = orders::OrderApi)
    )
)]
struct ApiDoc;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    logging::set();

    let api_config = envy::prefixed("RESTAURANT_API_")
        .from_env::<ApiConfig>()
        .unwrap();

    let db_config = envy::prefixed("RESTAURANT_DB_")
        .from_env::<DatabaseConfig>()
        .unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(db_config.max_connections)
        .connect_with(
            PgConnectOptions::new()
                .host(&db_config.host)
                .port(db_config.port)
                .username(&db_config.user)
                .database(&db_config.database)
                .password(&db_config.password)
        )
        .await
        .expect("Error building a connection pool");

    // sqlx::migrate!("db/migrations")
    //     .run(&pool)
    //     .await
    //     .unwrap();

    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        App::new()
            // Documentation
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone()),
            )
            // Context
            .app_data(
                Data::new(
                    AppContext{ db: pool.clone()}
                )
            )
            // Middlewares
            .wrap(middleware::Logger::default())
            // Api
            .service(web::scope("/orders").configure(orders::configure()))

    })
    .bind(api_config.host.to_owned())?
    .run()
    .await
}
