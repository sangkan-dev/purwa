//! Postgres-backed email/password authentication (optional `postgres` feature).
//!
//! Expects a `users` table with at least:
//! `id BIGINT PRIMARY KEY`, `email TEXT UNIQUE NOT NULL`, `password_hash TEXT NOT NULL`.

use axum_login::{AuthUser, AuthnBackend, UserId};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{PasswordError, password};

/// Row-shaped user for [`axum_login`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PgAuthUser {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
}

impl AuthUser for PgAuthUser {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash.as_bytes()
    }
}

/// Login credentials (email + plaintext password).
#[derive(Debug, Clone)]
pub struct EmailPasswordCredential {
    pub email: String,
    pub password: String,
}

/// SQLx [`AuthnBackend`] using `users` table.
#[derive(Debug, Clone)]
pub struct PgAuthnBackend {
    pool: PgPool,
}

impl PgAuthnBackend {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

impl AuthnBackend for PgAuthnBackend {
    type User = PgAuthUser;
    type Credentials = EmailPasswordCredential;
    type Error = sqlx::Error;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as::<_, PgAuthUserRow>(
            r#"SELECT id, email, password_hash FROM users WHERE email = $1"#,
        )
        .bind(&creds.email)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = user else {
            return Ok(None);
        };

        let ok = password::verify_password(&creds.password, &row.password_hash).unwrap_or(false);
        if !ok {
            return Ok(None);
        }

        Ok(Some(row.into()))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as::<_, PgAuthUserRow>(
            r#"SELECT id, email, password_hash FROM users WHERE id = $1"#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user.map(Into::into))
    }
}

#[derive(Debug, sqlx::FromRow)]
struct PgAuthUserRow {
    id: i64,
    email: String,
    password_hash: String,
}

impl From<PgAuthUserRow> for PgAuthUser {
    fn from(r: PgAuthUserRow) -> Self {
        Self {
            id: r.id,
            email: r.email,
            password_hash: r.password_hash,
        }
    }
}

/// Insert failures from hashing or SQL.
#[derive(Debug, thiserror::Error)]
pub enum InsertUserError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Password(#[from] PasswordError),
}

/// Insert a user with Argon2id-hashed password (application calls after validating email uniqueness).
pub async fn insert_user(
    pool: &PgPool,
    email: &str,
    plain_password: &str,
) -> Result<i64, InsertUserError> {
    let hash = password::hash_password(plain_password)?;

    let rec = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO users (email, password_hash)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind(email)
    .bind(&hash)
    .fetch_one(pool)
    .await?;

    Ok(rec)
}
