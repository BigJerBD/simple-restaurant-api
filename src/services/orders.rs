use actix_web::web::{Data, ServiceConfig};
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::error::ErrorKind;
use sqlx::{Error, FromRow};
use utoipa::{self, IntoParams, OpenApi, ToSchema};

use crate::app;

#[derive(OpenApi)]
#[openapi(
    paths(get_multiple, get_single, post_single, delete_single),
    components(schemas(Order, OrderCreateRequest, ErrorResponse))
)]
pub struct OrderApi;

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config
            .service(get_multiple)
            .service(get_single)
            .service(post_single)
            .service(delete_single);
    }
}

#[derive(PartialEq, Deserialize, Serialize, ToSchema, FromRow, Debug)]
struct Order {
    id: i32,
    table_number: i32,
    item_name: String,
    #[schema(value_type = String, format = DateTime)]
    ready_at: NaiveDateTime,
}

#[derive(Deserialize, Serialize, ToSchema, FromRow)]
struct OrderCreateRequest {
    table_number: i32,
    item_name: String,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct ListFilters {
    table_number: Option<i32>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct ErrorResponse {
    details: String,
}

#[utoipa::path(
    responses(
        (status = 200, body = Vec<Order>),
    ),
    params(ListFilters)
)]
#[get("/")]
pub async fn get_multiple(
    context: Data<app::State>,
    filters: web::Query<ListFilters>,
) -> impl Responder {
    let ListFilters { table_number } = filters.into_inner();

    match sqlx::query_as!(
        Order,
        r#"
        SELECT id, table_number, item_name, ready_at
        FROM restaurant_table_orders
        WHERE CASE
        WHEN $1::INT IS NOT NULL THEN table_number = $1
            ELSE TRUE
        END
         "#,
        table_number
    )
    .fetch_all(&context.db)
    .await
    {
        Ok(orders) => HttpResponse::Ok().json(orders),
        Err(e) => map_db_error_to_http(e),
    }
}

#[utoipa::path(
    responses(
        (status = 200, body = Vec<Order>),
        (status = 404, body = ErrorResponse),
    ),
)]
#[get("/{id}")]
pub async fn get_single(context: Data<app::State>, path: web::Path<i32>) -> impl Responder {
    let order_id = path.into_inner();

    match sqlx::query_as!(
        Order,
        "SELECT id, table_number, item_name, ready_at FROM restaurant_table_orders WHERE id = $1",
        order_id
    )
    .fetch_one(&context.db)
    .await
    {
        Ok(order) => HttpResponse::Ok().json(order),
        Err(e) => map_db_error_to_http(e),
    }
}

#[utoipa::path(
    request_body = OrderCreateRequest,
    responses(
        (status = 201, body = Order),
        (status = 404, body = ErrorResponse),
    ),
)]
#[post("/")]
pub async fn post_single(
    context: Data<app::State>,
    order_create_req: web::Json<OrderCreateRequest>,
) -> impl Responder {
    let request = order_create_req.into_inner();

    match sqlx::query_as!(
        Order,
        "INSERT INTO restaurant_table_orders (table_number,item_name, ready_at) VALUES ($1,$2, $3) RETURNING id, table_number, item_name, ready_at;",
        request.table_number,
        request.item_name,
        Utc::now().naive_utc() + chrono::Duration::minutes((rand::random::<i64>() % 10) + 5)
    )
        .fetch_one(&context.db)
        .await
    {
        Ok(order) => HttpResponse::Created().json(order),
        Err(e) => map_db_error_to_http(e)
    }
}

#[utoipa::path(
    responses(
        (status = 204),
        (status = 404, body = ErrorResponse),
    ),
)]
#[delete("/{id}")]
pub async fn delete_single(context: Data<app::State>, path: web::Path<i32>) -> impl Responder {
    let order_id = path.into_inner();

    match sqlx::query!(
        "DELETE FROM restaurant_table_orders WHERE id = $1 RETURNING id;",
        order_id
    )
    .fetch_one(&context.db)
    .await
    {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => map_db_error_to_http(e),
    }
}

fn map_db_error_to_http(error: Error) -> HttpResponse {
    match error {
        Error::RowNotFound => HttpResponse::NotFound().json(ErrorResponse {
            details: "Record Not found".to_string(),
        }),
        Error::Database(db_error) => match &db_error.kind() {
            ErrorKind::Other => HttpResponse::InternalServerError().json(ErrorResponse {
                details: "Internal Server Error".to_string(),
            }),
            _ => HttpResponse::UnprocessableEntity().json(ErrorResponse {
                details: db_error.to_string(),
            }),
        },
        _ => HttpResponse::InternalServerError().json(ErrorResponse {
            details: "Internal Server Error".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests_integration {
    use actix_web::{test, App};
    use chrono::NaiveDateTime;
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test(fixtures("orders"))]
    async fn test_gets_multiple(pool: PgPool) {
        pool.acquire().await.unwrap();
        let app = test::init_service(App::new().configure(app::configure(pool.clone()))).await;

        let req = test::TestRequest::get()
            .uri(&format!("{}/", app::orders_path()))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let json: Vec<Order> = test::read_body_json(resp).await;
        assert_eq!(
            json[0],
            Order {
                id: 100,
                table_number: 0,
                item_name: "test-poutine".to_string(),
                ready_at: NaiveDateTime::parse_from_str("2024-07-10 00:00:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap()
            }
        );
        assert_eq!(json.len(), 1);
    }
    #[sqlx::test(fixtures("orders"))]
    async fn test_gets_multiple_with_excluding_filter(pool: PgPool) {
        pool.acquire().await.unwrap();
        let app = test::init_service(App::new().configure(app::configure(pool.clone()))).await;

        let req = test::TestRequest::get()
            .uri(&format!("{}/?table_number=2222", app::orders_path()))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let json: Vec<Order> = test::read_body_json(resp).await;
        assert_eq!(json.len(), 0);
    }

    #[sqlx::test(fixtures("orders"))]
    async fn test_get_multiple_with_including_filter(pool: PgPool) {
        pool.acquire().await.unwrap();
        let app = test::init_service(App::new().configure(app::configure(pool.clone()))).await;

        let req = test::TestRequest::get()
            .uri(&format!("{}/?table_number=0", app::orders_path()))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let json: Vec<Order> = test::read_body_json(resp).await;
        assert_eq!(json.len(), 1);
    }

    #[sqlx::test(fixtures("orders"))]
    async fn test_get_single(pool: PgPool) {
        pool.acquire().await.unwrap();
        let app = test::init_service(App::new().configure(app::configure(pool.clone()))).await;

        let req = test::TestRequest::get()
            .uri(&format!("{}/100", app::orders_path()))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let json: Order = test::read_body_json(resp).await;
        assert_eq!(
            json,
            Order {
                id: 100,
                table_number: 0,
                item_name: "test-poutine".to_string(),
                ready_at: NaiveDateTime::parse_from_str("2024-07-10 00:00:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap()
            }
        );
    }

    #[sqlx::test(fixtures("orders"))]
    async fn test_get_single_not_found(pool: PgPool) {
        pool.acquire().await.unwrap();
        let app = test::init_service(App::new().configure(app::configure(pool.clone()))).await;

        let req = test::TestRequest::get()
            .uri(&format!("{}/999", app::orders_path()))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        let _json: ErrorResponse = test::read_body_json(resp).await;
    }

    #[sqlx::test(fixtures("orders"))]
    async fn test_delete_single(pool: PgPool) {
        pool.acquire().await.unwrap();
        let app = test::init_service(App::new().configure(app::configure(pool.clone()))).await;

        let req = test::TestRequest::delete()
            .uri(&format!("{}/100", app::orders_path()))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 204);

        // Todo: Ideally also check the DB content
    }

    #[sqlx::test(fixtures("orders"))]
    async fn test_delete_single_not_found(pool: PgPool) {
        pool.acquire().await.unwrap();
        let app = test::init_service(App::new().configure(app::configure(pool.clone()))).await;

        let req = test::TestRequest::delete()
            .uri(&format!("{}/999", app::orders_path()))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        let _json: ErrorResponse = test::read_body_json(resp).await;

        // Todo: Ideally also check the DB content
    }

    #[sqlx::test(fixtures("orders"))]
    async fn test_post_single(pool: PgPool) {
        pool.acquire().await.unwrap();
        let app = test::init_service(App::new().configure(app::configure(pool.clone()))).await;

        let req = test::TestRequest::post()
            .uri(&format!("{}/", app::orders_path()))
            .set_json(OrderCreateRequest {
                table_number: 1,
                item_name: "test".to_string(),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let json: Order = test::read_body_json(resp).await;
        assert_eq!(json.table_number, 1);
        assert_eq!(json.item_name, "test");

        // Todo: Ideally also check the DB content
    }

    #[sqlx::test(fixtures("orders"))]
    async fn test_post_single_invalid_table(pool: PgPool) {
        pool.acquire().await.unwrap();
        let app = test::init_service(App::new().configure(app::configure(pool.clone()))).await;

        let req = test::TestRequest::post()
            .uri(&format!("{}/", app::orders_path()))
            .set_json(OrderCreateRequest {
                table_number: 10000000,
                item_name: "test".to_string(),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 422);

        let _json: ErrorResponse = test::read_body_json(resp).await;

        // Todo: Ideally also check the DB content
    }
}
