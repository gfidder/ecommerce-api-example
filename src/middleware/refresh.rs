use axum::{
    RequestPartsExt,
    extract::{Request, State},
    http::{
        HeaderValue,
        header::{AUTHORIZATION, SET_COOKIE},
    },
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    extract::cookie,
    headers::{Authorization, Cookie, authorization::Bearer},
    typed_header::TypedHeaderRejectionReason,
};
use redis::AsyncCommands as _;

use crate::{AppState, auth::AuthError, error::Result, models::TokenDetails};

pub async fn refresh_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response> {
    let (mut parts, body) = request.into_parts();

    let refresh_token = match parts.extract::<TypedHeader<Cookie>>().await {
        Ok(cookies) => cookies.get("refresh_token").map(ToString::to_string),
        Err(_) => return Ok(AuthError::MissingCredentials.into_response()),
    };

    let Some(refresh_token) = refresh_token else {
        return Ok(AuthError::MissingCredentials.into_response());
    };

    let access_token = match parts.extract::<TypedHeader<Authorization<Bearer>>>().await {
        Ok(header) => Some(header.token().to_string()),
        Err(err) => {
            if matches!(err.reason(), TypedHeaderRejectionReason::Missing) {
                parts.extract::<TypedHeader<Cookie>>().await.ok().and_then(
                    |TypedHeader(cookies)| cookies.get("access_token").map(ToString::to_string),
                )
            } else {
                return Ok(AuthError::InvalidToken.into_response());
            }
        }
    };

    let refresh_token_details = match state.auth.refresh.verify_token(&refresh_token) {
        Ok(details) => details,
        Err(err) => return Ok(err.into_response()),
    };

    // Check if the refresh token is in cache
    let mut redis_conn = state.redis.clone();
    let redis_key = format!("refresh_token:{}", refresh_token_details.token_id);

    let stored_token: Result<String, crate::error::Error> = redis_conn
        .get(&redis_key)
        .await
        .map_err(crate::error::Error::Redis);

    let stored_details = match stored_token {
        Ok(token) => match serde_json::from_str::<TokenDetails>(&token)
            .map_err(crate::error::Error::SerdeJson)
        {
            Ok(details) => details,
            Err(err) => return Ok(err.response()),
        },
        Err(err) => return Ok(err.response()),
    };

    // if access token is missing we issue a new one
    let new_access_token: String;
    if let Some(token) = access_token {
        match state.auth.access.verify_token(&token) {
            Ok(_) => new_access_token = token,
            Err(_) => {
                // Token is invalid for whatever reason
                match state.auth.access.generate_token(stored_details.user_pid) {
                    Ok(details) => new_access_token = details.token.unwrap(),
                    Err(e) => return Ok(e.into_response()),
                }
            }
        }
    } else {
        // No access token present; probably expired and got expelled from cookies
        match state.auth.access.generate_token(stored_details.user_pid) {
            Ok(details) => {
                new_access_token = details.token.unwrap();
            }
            Err(e) => return Ok(e.into_response()),
        }
    }

    let access_cookie = cookie::Cookie::build(("access_token", &new_access_token))
        .path("/")
        .max_age(time::Duration::seconds(state.auth.access.exp))
        .same_site(cookie::SameSite::Lax)
        .http_only(true)
        .to_string();

    let mut req = Request::from_parts(parts, body);
    req.headers_mut().append(
        AUTHORIZATION,
        HeaderValue::from_str(format!("Bearer: {}", &new_access_token).as_str()).unwrap(),
    );
    req.headers_mut().append(
        SET_COOKIE,
        HeaderValue::from_str(access_cookie.as_str()).unwrap(),
    );

    Ok(next.run(req).await)
}
