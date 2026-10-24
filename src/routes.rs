use axum::Router;

pub fn routes() -> Router {
    Router::new().route("/health", axum::routing::get(|| async { "healthy" }))
}
