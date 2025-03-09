use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::state::{add_sni_endpoint, get_sni_endpoints, remove_sni_endpoint};

pub async fn start(listen_addr: &SocketAddr) -> anyhow::Result<()> {
    // Define the routes
    let app = Router::new()
        .route("/sni", get(get_sni))
        .route("/sni", put(put_sni))
        .route("/sni/{name}", delete(delete_sni))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    // Define the address to bind the server to

    let listener = tokio::net::TcpListener::bind(listen_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[utoipa::path(get, path = "/sni")]
async fn get_sni() -> impl IntoResponse {
    Json(get_sni_endpoints().await)
}

#[utoipa::path(
    put,
    path = "/sni",
    request_body = Sni,
    responses(
        (status = 200, description = "Echoed message", body = Sni)
    )
)]
async fn put_sni(Json(payload): Json<Sni>) -> impl IntoResponse {
    let Ok(dest) = payload.destination.parse() else {
        return StatusCode::BAD_REQUEST;
    };

    add_sni_endpoint(payload.sni, dest).await;

    StatusCode::OK
}

#[utoipa::path(
    delete,
    path = "/sni/{name}",
    params(
        ("name" = String, Path, description = "SNI to remove")
    ),
)]
async fn delete_sni(Path(name): Path<String>) -> impl IntoResponse {
    remove_sni_endpoint(&name).await;

    StatusCode::OK
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
struct Sni {
    sni: String,

    /// Should be a socket address
    destination: String,
}

// Define the OpenAPI documentation
#[derive(OpenApi)]
#[openapi(paths(get_sni, put_sni, delete_sni), components(schemas(Sni)))]
struct ApiDoc;
