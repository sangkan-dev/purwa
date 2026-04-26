//! Session + axum-login smoke test (memory store, in-memory backend).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Form, Router};
use axum_login::{AuthManagerLayerBuilder, AuthSession, AuthUser, AuthnBackend, UserId};
use http_body_util::BodyExt;
use serde::Deserialize;
use serde_json::Value;
use tower::ServiceExt;

use purwa_auth::{CurrentUser, memory_session_layer};

#[derive(Debug, Clone)]
struct User {
    id: i64,
    pw_hash: String,
}

impl AuthUser for User {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.pw_hash.as_bytes()
    }
}

#[derive(Clone, Default)]
struct Backend {
    users: Arc<Mutex<HashMap<i64, User>>>,
}

#[derive(Clone, Deserialize)]
struct Creds {
    user_id: i64,
}

impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Creds;
    type Error = std::convert::Infallible;

    async fn authenticate(
        &self,
        Creds { user_id }: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        Ok(self.users.lock().unwrap().get(&user_id).cloned())
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(self.users.lock().unwrap().get(user_id).cloned())
    }
}

async fn login(mut auth: AuthSession<Backend>, Form(creds): Form<Creds>) -> impl IntoResponse {
    let u = match auth.authenticate(creds).await {
        Ok(Some(u)) => u,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    if auth.login(&u).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    StatusCode::OK.into_response()
}

async fn me(CurrentUser(u): CurrentUser<Backend>) -> impl IntoResponse {
    format!("id={}", u.id)
}

#[tokio::test]
async fn login_then_current_user() {
    let backend = Backend::default();
    backend.users.lock().unwrap().insert(
        1,
        User {
            id: 1,
            pw_hash: "x".into(),
        },
    );

    let session_layer = memory_session_layer();
    let auth_layer = AuthManagerLayerBuilder::new(backend.clone(), session_layer).build();

    let app = Router::new()
        .route("/login", post(login))
        .route("/me", get(me))
        .layer(auth_layer);

    let req = Request::builder()
        .method("POST")
        .uri("/login")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("user_id=1"))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let set_cookie = res
        .headers()
        .get_all(axum::http::header::SET_COOKIE)
        .iter()
        .next()
        .expect("session cookie")
        .to_str()
        .unwrap()
        .to_string();

    let req = Request::builder()
        .method("GET")
        .uri("/me")
        .header(
            axum::http::header::COOKIE,
            extract_cookie_value(&set_cookie),
        )
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(body.as_ref(), b"id=1");
}

#[tokio::test]
async fn me_without_cookie_returns_401_purwa_error_json() {
    let backend = Backend::default();
    let session_layer = memory_session_layer();
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let app = Router::new().route("/me", get(me)).layer(auth_layer);

    let req = Request::builder()
        .method("GET")
        .uri("/me")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["message"], "login required");
}

fn extract_cookie_value(set_cookie: &str) -> String {
    // "name=value; Path=/; ..."
    set_cookie
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}
