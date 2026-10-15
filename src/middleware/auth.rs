use axum::{
    RequestPartsExt,
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, Cookie, authorization::Bearer},
    typed_header::TypedHeaderRejectionReason,
};

use crate::{AppState, auth::AuthError, error::Result};

pub async fn auth_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response> {
    let (mut parts, body) = request.into_parts();

    let access_token = match parts.extract::<TypedHeader<Authorization<Bearer>>>().await {
        Ok(header) => Some(header.token().to_string()),
        Err(err) => {
            if matches!(err.reason(), TypedHeaderRejectionReason::Missing) {
                parts.extract::<TypedHeader<Cookie>>().await.ok().and_then(
                    |TypedHeader(cookies)| cookies.get("access_token").map(ToString::to_string),
                )
            } else {
                return Ok(AuthError::InvalidToken.response());
            }
        }
    };

    let Some(access_token) = access_token else {
        return Ok(AuthError::MissingCredentials.response());
    };

    let _token_details = match state.auth.access.verify_token(&access_token) {
        Ok(details) => details,
        Err(err) => return Ok(err.into_response()),
    };

    let req = Request::from_parts(parts, body);

    Ok(next.run(req).await)
}
