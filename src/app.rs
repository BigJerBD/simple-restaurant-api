use actix_web::web;
use actix_web::web::{Data, ServiceConfig};
use sqlx::{Pool, Postgres};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::services::orders;

pub fn orders_path() -> String {
    "/orders".to_string()
}

pub struct State {
    pub db: Pool<Postgres>,
}

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/orders", api = orders::OrderApi)
    )
)]
struct ApiDoc;

pub fn configure(pool: Pool<Postgres>) -> impl FnOnce(&mut ServiceConfig) {
    let openapi = ApiDoc::openapi();

    move |config: &mut ServiceConfig| {
        config
            // Documentation
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            // State
            .app_data(Data::new(State { db: pool.clone() }))
            // Api
            .service(web::scope(&orders_path()).configure(orders::configure()));
    }
}

#[cfg(test)]
mod tests_integration {
    use actix_web::{test, App};
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn test_swagger_ui(pool: PgPool) {
        pool.acquire().await.unwrap();
        let app = test::init_service(App::new().configure(configure(pool.clone()))).await;

        let req = test::TestRequest::get()
            .uri("/swagger-ui/index.html")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }
}
