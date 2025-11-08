// TODO: Implement auth middleware
use axum::{extract::Request, middleware::Next, response::Response};
use std::sync::Arc;

use crate::handlers::CategoryHandlerState;

pub async fn auth_middleware(request: Request, next: Next) -> Response {
    // TODO: Implement authentication middleware
    next.run(request).await
}

pub async fn state_middleware(
    state: Arc<CategoryHandlerState>,
    mut request: Request,
    next: Next,
) -> Response {
    // Insert the state into request extensions
    request.extensions_mut().insert(state);
    next.run(request).await
}
