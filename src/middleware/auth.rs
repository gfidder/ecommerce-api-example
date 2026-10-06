use axum::{
    body::Body,
    extract::{Json, Request},
    http,
    http::{Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};

use crate::auth::AuthError;

pub async fn authorization_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response<Body>, AuthError> {
    let auth_header = request.headers_mut().get(http::header::AUTHORIZATION);
    let auth_header = match auth_header {
        Some(header) => header.to_str().map_err(|_| AuthError::WrongCredentials)?,
        None => return Err(AuthError::MissingCredentials),
    };
    let mut header = auth_header.split_whitespace();
    let (bearer, token) = (header.next(), header.next());
    // let token_data = match

    Ok(next.run(request).await)
}
