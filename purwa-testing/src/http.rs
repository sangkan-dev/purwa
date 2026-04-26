//! Tower/Axum one-shot helpers for integration-style tests.

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::response::Response;
use bytes::Bytes;
use http_body_util::BodyExt;
use tower::ServiceExt;

/// Sends one HTTP request through the router and returns the response.
///
/// For Axum’s default router the service error is [`std::convert::Infallible`]; this should not panic.
pub async fn oneshot(router: Router, request: Request<Body>) -> Response {
    router.oneshot(request).await.unwrap()
}

/// Builds a GET (empty body) request, runs [`oneshot`], returns the status code.
pub async fn oneshot_status(router: Router, uri: &str) -> StatusCode {
    let req = Request::builder()
        .method(Method::GET)
        .uri(uri)
        .body(Body::empty())
        .expect("valid request");
    oneshot(router, req).await.status()
}

/// Same as [`oneshot_status`] but allows choosing the method.
pub async fn oneshot_status_with_method(router: Router, method: Method, uri: &str) -> StatusCode {
    let req = Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .expect("valid request");
    oneshot(router, req).await.status()
}

/// Collects the full response body as bytes (useful for assertions).
pub async fn oneshot_body_bytes(router: Router, uri: &str) -> Bytes {
    let req = Request::builder()
        .method(Method::GET)
        .uri(uri)
        .body(Body::empty())
        .expect("valid request");
    let res = oneshot(router, req).await;
    response_body_bytes(res).await.expect("read body")
}

/// Reads the entire body after the response has been produced.
pub async fn response_body_bytes(response: Response) -> Result<Bytes, std::io::Error> {
    let collected = response
        .into_body()
        .collect()
        .await
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    Ok(collected.to_bytes())
}

/// Error from [`json_body`].
#[derive(Debug)]
pub enum JsonBodyError {
    Body(std::io::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for JsonBodyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonBodyError::Body(e) => write!(f, "body: {e}"),
            JsonBodyError::Json(e) => write!(f, "json: {e}"),
        }
    }
}

impl std::error::Error for JsonBodyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            JsonBodyError::Body(e) => Some(e),
            JsonBodyError::Json(e) => Some(e),
        }
    }
}

/// Collects the body and parses it as JSON.
pub async fn json_body(response: Response) -> Result<serde_json::Value, JsonBodyError> {
    let bytes = response_body_bytes(response)
        .await
        .map_err(JsonBodyError::Body)?;
    serde_json::from_slice(&bytes).map_err(JsonBodyError::Json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::get;

    #[tokio::test]
    async fn oneshot_hits_route() {
        let app = Router::new().route("/hi", get(|| async { "hello" }));
        let status = oneshot_status(app, "/hi").await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn oneshot_body_bytes_round_trip() {
        let app = Router::new().route("/x", get(|| async { "payload" }));
        let bytes = oneshot_body_bytes(app, "/x").await;
        assert_eq!(&bytes[..], b"payload");
    }

    #[tokio::test]
    async fn json_body_parses() {
        let app = Router::new().route(
            "/j",
            get(|| async { axum::Json(serde_json::json!({ "a": 1 })) }),
        );
        let req = Request::builder().uri("/j").body(Body::empty()).unwrap();
        let res = oneshot(app, req).await;
        let v = json_body(res).await.expect("json");
        assert_eq!(v["a"], 1);
    }
}
