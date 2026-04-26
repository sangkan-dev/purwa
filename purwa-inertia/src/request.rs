//! Inertia request metadata from headers.

use std::collections::HashSet;

use axum::body::Body;
use axum::extract::FromRequestParts;
use axum::http::header::{self, HeaderMap, HeaderValue};
use axum::http::request::Parts;
use axum::http::{Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use serde_json::{Map, Value};

use crate::headers::{
    X_INERTIA, X_INERTIA_PARTIAL_COMPONENT, X_INERTIA_PARTIAL_DATA, X_INERTIA_PARTIAL_EXCEPT,
    X_INERTIA_VERSION,
};

/// Request-scoped values for [`InertiaRequest::respond`] (keeps the handler signature small).
#[derive(Debug, Clone, Copy)]
pub struct InertiaRenderContext<'a> {
    pub method: &'a Method,
    pub request_uri: &'a Uri,
    pub host_header: Option<&'a str>,
    pub asset_version: &'a str,
    /// Full-page HTML only: raw HTML fragment (Vite `<script>` / `<link>` tags) inserted before `</body>`.
    pub html_body_injection: Option<&'a str>,
}

/// Parsed Inertia-related headers for one request.
#[derive(Debug, Clone, Default)]
pub struct InertiaRequest {
    /// `X-Inertia: true`
    pub is_inertia: bool,
    pub client_version: Option<String>,
    pub partial_component: Option<String>,
    pub partial_data: Option<Vec<String>>,
    pub partial_except: Option<Vec<String>>,
}

impl InertiaRequest {
    fn parse_headers(headers: &HeaderMap) -> Self {
        let is_inertia = headers
            .get(X_INERTIA)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|s| s.eq_ignore_ascii_case("true"));

        let client_version = headers
            .get(X_INERTIA_VERSION)
            .and_then(|v| v.to_str().ok())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from);

        let partial_component = headers
            .get(X_INERTIA_PARTIAL_COMPONENT)
            .and_then(|v| v.to_str().ok())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from);

        let partial_data = parse_comma_list(headers, X_INERTIA_PARTIAL_DATA);
        let partial_except = parse_comma_list(headers, X_INERTIA_PARTIAL_EXCEPT);

        Self {
            is_inertia,
            client_version,
            partial_component,
            partial_data,
            partial_except,
        }
    }

    /// `409 Conflict` with `X-Inertia-Location` when asset versions differ (**GET** Inertia requests only; protocol v1.3).
    pub fn version_conflict_response(
        &self,
        method: &Method,
        server_version: &str,
        request_uri: &Uri,
        host_header: Option<&str>,
    ) -> Option<Response> {
        if !self.is_inertia || *method != Method::GET {
            return None;
        }
        let client = self.client_version.as_deref()?;
        if client == server_version {
            return None;
        }
        let location = full_location_url(request_uri, host_header);
        let mut res = Response::builder()
            .status(StatusCode::CONFLICT)
            .body(Body::empty())
            .ok()?;
        res.headers_mut().insert(
            header::HeaderName::from_static("x-inertia-location"),
            HeaderValue::try_from(location).ok()?,
        );
        Some(res)
    }

    /// Build an Inertia page response (JSON or HTML skeleton) or a version-conflict response.
    ///
    /// * **`shared`**: merged under page props first; handler props override on key clash.
    /// * **Partial reloads**: when `X-Inertia-Partial-Component` matches `component`, props are
    ///   filtered per `X-Inertia-Partial-Data` / `X-Inertia-Partial-Except` (except wins if both set).
    ///   The `errors` key is always kept.
    pub fn respond(
        &self,
        ctx: &InertiaRenderContext<'_>,
        component: &str,
        page_props: impl serde::Serialize,
        shared: &crate::SharedProps,
    ) -> Result<Response, serde_json::Error> {
        if let Some(conflict) = self.version_conflict_response(
            ctx.method,
            ctx.asset_version,
            ctx.request_uri,
            ctx.host_header,
        ) {
            return Ok(conflict);
        }

        let mut props = merge_props(&shared.0, serde_json::to_value(&page_props)?);
        ensure_errors_prop(&mut props);
        apply_partial_filter(self, component, &mut props);

        let url = page_url(ctx.request_uri);
        let page = serde_json::json!({
            "component": component,
            "props": Value::Object(props),
            "url": url,
            "version": ctx.asset_version,
        });

        if self.is_inertia {
            return Ok(json_inertia_response(&page));
        }

        Ok(html_shell_response(
            &page,
            ctx.html_body_injection.unwrap_or(""),
        ))
    }
}

fn parse_comma_list(headers: &HeaderMap, name: &str) -> Option<Vec<String>> {
    let raw = headers.get(name)?.to_str().ok()?;
    let list: Vec<String> = raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();
    if list.is_empty() { None } else { Some(list) }
}

fn merge_props(shared: &Map<String, Value>, page: Value) -> Map<String, Value> {
    let mut out = shared.clone();
    if let Value::Object(p) = page {
        for (k, v) in p {
            out.insert(k, v);
        }
    }
    out
}

fn ensure_errors_prop(props: &mut Map<String, Value>) {
    props
        .entry("errors".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
}

fn apply_partial_filter(inertia: &InertiaRequest, component: &str, props: &mut Map<String, Value>) {
    if inertia.partial_component.as_deref() != Some(component) {
        return;
    }

    let except: HashSet<&str> = inertia
        .partial_except
        .iter()
        .flatten()
        .map(|s| s.as_str())
        .collect();
    let data: HashSet<&str> = inertia
        .partial_data
        .iter()
        .flatten()
        .map(|s| s.as_str())
        .collect();

    if !except.is_empty() {
        props.retain(|k, _| k == "errors" || !except.contains(k.as_str()));
    } else if !data.is_empty() {
        props.retain(|k, _| k == "errors" || data.contains(k.as_str()));
    }
}

fn page_url(uri: &Uri) -> String {
    uri.path_and_query()
        .map(|pq| pq.as_str().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "/".to_string())
}

fn full_location_url(uri: &Uri, host_header: Option<&str>) -> String {
    if uri.scheme().is_some() {
        return uri.to_string();
    }
    let host = host_header.unwrap_or("localhost");
    let path_q = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
    format!("http://{host}{path_q}")
}

fn json_inertia_response(page: &Value) -> Response {
    let mut res = axum::Json(page.clone()).into_response();
    res.headers_mut().insert(
        header::HeaderName::from_static("x-inertia"),
        HeaderValue::from_static("true"),
    );
    res.headers_mut().insert(
        header::HeaderName::from_static("vary"),
        HeaderValue::from_static("X-Inertia"),
    );
    res
}

fn html_shell_response(page: &Value, body_injection: &str) -> Response {
    let json = page.to_string();
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Purwa + Inertia</title>
</head>
<body>
  <script type="application/json" data-page="app">{json}</script>
  <div id="app"></div>
  {body_injection}
</body>
</html>"#
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(html))
        .unwrap()
}

impl<S> FromRequestParts<S> for InertiaRequest
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(InertiaRequest::parse_headers(&parts.headers))
    }
}
