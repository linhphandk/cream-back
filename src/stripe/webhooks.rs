use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use http_body_util::BodyExt;
use serde::Serialize;
use serde_json::Value;
use crate::shared::errors::{AppError, AppResult};
use crate::stripe::client::StripeClient;

/// Extract and verify Stripe webhook signature
pub async fn verify_stripe_signature(
    State(client): State<StripeClient>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> AppResult<Value> {
    let signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::BadRequest("Missing stripe-signature header".to_string()))?;

    let event = client.verify_webhook(&body, signature)?;
    Ok(event)
}

/// Error response for webhook middleware
#[derive(Serialize)]
struct WebhookError {
    error: String,
}

impl IntoResponse for WebhookError {
    fn into_response(self) -> Response {
        let mut response = Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&self).unwrap()))
            .unwrap();
        response
    }
}

/// Middleware to verify Stripe webhook signature
pub async fn stripe_webhook_middleware(
    State(client): State<StripeClient>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let headers = request.headers().clone();
    let body = request.body_mut();
    let body_bytes = body.collect().await
        .map_err(|_| WebhookError { error: "Failed to read body".to_string() }.into_response())?
        .to_bytes();

    let signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| WebhookError { error: "Missing stripe-signature header".to_string() }.into_response())?;

    let event = client.verify_webhook(&body_bytes, signature)
        .map_err(|e| WebhookError { error: e.to_string() }.into_response())?;

    // Store verified event in request extensions
    request.extensions_mut().insert(event);
    
    // Rebuild request with body
    let mut new_request = Request::builder()
        .method(request.method().clone())
        .uri(request.uri().clone())
        .version(request.version())
        .body(Body::from(body_bytes))
        .unwrap();
    *new_request.headers_mut() = headers;
    *new_request.extensions_mut() = request.extensions().clone();

    Ok(next.run(new_request).await)
}
