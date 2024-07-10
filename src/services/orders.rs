use actix_web::{delete, get, HttpResponse, post, Responder, web};
use actix_web::web::{Data, get, ServiceConfig};
use serde::{Deserialize, Serialize, Serializer};
use sqlx::{Error, FromRow};
use sqlx::types::Uuid;
use utoipa::{OpenApi, ToSchema, self, IntoParams};
use crate::AppContext;
use crate::services::orders;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_multiple,
        get_single,
        post_single,
        delete_single
    ),
    components(schemas(Order, OrderCreateRequest, ErrorResponse)),
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

#[derive(Serialize, ToSchema, FromRow)]
struct Order {
    id: i32,
    table_number: i32,
    item_name: String,
}

#[derive(Deserialize, ToSchema, FromRow)]
struct OrderCreateRequest {
    table_number: i32,
    item_name: String
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct ListFilters {
    table_number: i32,
}

#[derive(Serialize, ToSchema)]
pub struct  ErrorResponse {
    details: String
}

#[utoipa::path(
    responses(
        (status = 200, body = Vec<Order>),
    ),
    params(ListFilters)
)]
#[get("/")]
pub async fn get_multiple(
    context: Data<AppContext>,
    filters: web::Query<ListFilters>
) -> impl Responder {
    let ListFilters{ table_number } = filters.into_inner();

    match sqlx::query_as!(
        Order,
        "SELECT id, table_number, item_name FROM restaurant_table_orders WHERE table_number = $1;",
        table_number
    )
        .fetch_all(&context.db)
        .await
    {
        Ok(orders) => HttpResponse::Ok().json(orders),
        Err(e) => map_db_error_to_http(e)
    }
}


#[utoipa::path(
    responses(
        (status = 200, body = Vec<Order>),
        (status = 404, body = ErrorResponse),
    ),
)]
#[get("/{id}")]
pub async fn get_single(
    context: Data<AppContext>,
    path: web::Path<i32>,
) -> impl Responder {
    let order_id = path.into_inner();

    match sqlx::query_as!(
        Order,
        "SELECT id, table_number, item_name FROM restaurant_table_orders WHERE id = $1",
        Some(order_id)
    )
        .fetch_one(&context.db)
        .await
    {
        Ok(order) => HttpResponse::Ok().json(order),
        Err(e) => map_db_error_to_http(e)
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
    context: Data<AppContext>,
    order_create_req: web::Json<OrderCreateRequest>,
) -> impl Responder {
    let request = order_create_req.into_inner();

    match sqlx::query_as!(
        Order,
        "INSERT INTO restaurant_table_orders (table_number,item_name) VALUES ($1,$2) RETURNING id, table_number, item_name;",
        request.table_number,
        request.item_name
    )
        .fetch_all(&context.db)
        .await
    {
        Ok(order) => HttpResponse::Ok().json(order),
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
pub async fn delete_single(
    context: Data<AppContext>,
    path: web::Path<i32>,
) -> impl Responder {
    let  order_id= path.into_inner();

    match sqlx::query!(
        "DELETE FROM restaurant_table_orders WHERE id = $1",
        order_id
    )
        .execute(&context.db)
        .await
    {
        Ok(order) => HttpResponse::NoContent().finish(),
        Err(e) => map_db_error_to_http(e),
    }
}


fn map_db_error_to_http(error: Error) -> HttpResponse {
    match error {
        Error::RowNotFound => HttpResponse::NotFound().json(ErrorResponse { details: "Record Not found".to_string()}),
        _ => HttpResponse::InternalServerError().json(ErrorResponse { details: "Internal Server Error".to_string()}),
    }
}
