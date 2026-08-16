//! Discovery aid for routes Atlas has not implemented.
//!
//! The endpoint list here was assembled from documentation and from other
//! Jellyfin-compatible servers, not from watching Infuse. This fallback turns
//! that guesswork into observation: point a real client at a permissive build
//! and the log names every route it wanted.
//!
//! In permissive mode an unmatched route answers `200 {}` rather than 404,
//! because some clients abandon a whole flow on a single unexpected 404. That is
//! a debugging posture, not a production one — with the flag off, unmatched
//! routes 404 honestly and are still logged.

use axum::{extract::Request, http::StatusCode, response::IntoResponse, response::Response, Json};

/// Query parameters that may carry a credential. Jellyfin clients accept
/// `?api_key=` on image and stream URLs, and telemetry must never see it.
const SENSITIVE_PARAMS: [&str; 3] = ["api_key", "apikey", "token"];

pub async fn unmatched(request: Request) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let query = request.uri().query().map(redact_query).unwrap_or_default();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();

    tracing::warn!(
        jellyfin_unmatched = true,
        %method,
        %path,
        %query,
        %user_agent,
        "Jellyfin route not implemented"
    );

    if super::permissive() {
        (StatusCode::OK, Json(serde_json::json!({}))).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "Error": "NotFound", "Path": path })),
        )
            .into_response()
    }
}

/// Keeps parameter names, drops any value that could be a credential.
pub fn redact_query(query: &str) -> String {
    query
        .split('&')
        .map(|pair| match pair.split_once('=') {
            Some((key, _)) if SENSITIVE_PARAMS.contains(&key.to_ascii_lowercase().as_str()) => {
                format!("{key}=[redacted]")
            }
            _ => pair.to_string(),
        })
        .collect::<Vec<_>>()
        .join("&")
}

#[cfg(test)]
mod tests {
    use super::redact_query;

    #[test]
    fn redacts_credentials_from_logged_queries() {
        assert_eq!(
            redact_query("api_key=secret&ParentId=abc"),
            "api_key=[redacted]&ParentId=abc"
        );
        assert_eq!(redact_query("Token=secret"), "Token=[redacted]");
    }

    #[test]
    fn leaves_ordinary_parameters_readable() {
        // The point of the log is to see what a client asked for, so everything
        // that is not a credential has to survive.
        assert_eq!(
            redact_query("StartIndex=0&Limit=50&IncludeItemTypes=Movie"),
            "StartIndex=0&Limit=50&IncludeItemTypes=Movie"
        );
        assert_eq!(redact_query(""), "");
    }
}
