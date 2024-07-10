use actix_web::{HttpResponse, Responder, web};
use actix_web::web::{Data, ServiceConfig};
use serde::{Deserialize, Serialize, Serializer};
use sqlx::{Error, FromRow};
use sqlx::types::Uuid;
use utoipa::{OpenApi, ToSchema, self, IntoParams};
use crate::AppContext;
use crate::services::orders;

#[derive(OpenApi)]
#[openapi(
    paths(
        list,
        get,
        post,
        delete,
    ),
    components(schemas(Order, OrderCreateRequest))
)]
pub struct OrderApi;

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config
            .route("/", web::get().to(list))
            .route("/", web::get().to(post))
            .route("/{id}", web::get().to(get))
            .route("/{id}", web::get().to(delete));
    }
}


#[derive(Serialize, ToSchema, FromRow)]
struct Order {
    id: i32,
    table_number: i32,
    item_name: String
}

#[derive(Serialize, ToSchema, FromRow)]
struct OrderCreateRequest {
    table_number: i32,
    item_name: String
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct ListFilters {
    table_number: i32,
}

pub enum ErrorResponse {
    NotFound(String),
}

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, body = Vec<Order>),
    ),
    params(ListFilters)
)]
pub async fn list(
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
    get,
    path = "/{id}",
    responses(
        (status = 200, body = Vec<Order>),
        (status = 404, body = ErrorResponse),
    ),
    params(
        ("id", description = "Unique id for the order")
    ),
)]
pub async fn get(
    context: Data<AppContext>,
    path: web::Path<i32>,
) -> impl Responder {
    let order_id = path.into_inner();

    match `sqlx::query_as!(
        Order,
        "SELECT id, table_number, item_name FROM restaurant_table_orders WHERE id = $1",
        order_id
    )
        .fetch_one(&context.db)
        .await
    {
        Ok(order) => HttpResponse::Ok().json(order),
        Err(e) => map_db_error_to_http(e)
    }
}

#[utoipa::path(
    post,
    path = "/",
    request_body = OrderCreateRequest,
    responses(
        (status = 201, body = Order),
        (status = 404, body = ErrorResponse),
    ),
)]
pub async fn post(
    context: Data<AppContext>,

) -> impl Responder {
    let order_id= path.into_inner();

    match sqlx::query_as!(
        Order,
        "INSERT INTO restaurant_table_orders (table_name,item_name) VALUES ($1,$3) returning id",
        table_number,
        order_id
    )
        .fetch_all(&context.db)
        .await
    {
        Ok(order) => HttpResponse::Ok().json(order),
        Err(e) => map_db_error_to_http(e)
    }
}

#[utoipa::path(
    delete,
    path = "/{id}",
    responses(
        (status = 204),
        (status = 404, body = ErrorResponse),
    ),
    params(
        ("id", description = "Unique id for the order")
    ),
)]
pub async fn delete(
    context: Data<AppContext>,
    path: web::Path<i32>,
) -> impl Responder {
    let  order_id= path.into_inner();

    match sqlx::query!(
        "DELETE FROM restaurant_table_orders WHERE id = $1",
        order_id
    )
        //fetch one?
        .execute(&context.db)
        .await
    {
        Ok(order) => HttpResponse::NoContent().finish(),
        Err(e) => map_db_error_to_http(e),
    }
}


fn map_db_error_to_http(error: Error) -> HttpResponse {
    match error {
        Error::RowNotFound => HttpResponse::NotFound().finish(),
        _ => HttpResponse::InternalServerError().finish()
    }
}