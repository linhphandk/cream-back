use axum::Router;

pub fn routes() -> Router {
    Router::new()
        .route("/api/orgs", axum::routing::post(|| async { "org created" }))
        .route("/api/orgs/:subdomain", axum::routing::get(|| async { "org by subdomain" }))
}