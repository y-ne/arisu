use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use sqlx::PgPool;
use tower_sessions::Session;
use uuid::Uuid;

use crate::models::user::User;

const USER_ID_KEY: &str = "user_id";

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub display_name: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn register(
    State(pool): State<PgPool>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<User>), StatusCode> {
    if req.username.trim().is_empty()
        || req.display_name.trim().is_empty()
        || req.password.len() < 8
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let password_hash = hash_password(&req.password)?;
    let id = Uuid::now_v7();

    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (id, username, display_name, password_hash)
        VALUES ($1, $2, $3, $4)
        RETURNING
            id, username, display_name, password_hash,
            role AS "role: _",
            created_at, updated_at, deleted_at
        "#,
        id,
        req.username,
        req.display_name,
        password_hash,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => StatusCode::CONFLICT,
        _ => {
            eprintln!("register db error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn login(
    State(pool): State<PgPool>,
    session: Session,
    Json(req): Json<LoginRequest>,
) -> Result<Json<User>, StatusCode> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT
            id, username, display_name, password_hash,
            role AS "role: _",
            created_at, updated_at, deleted_at
        FROM users
        WHERE username = $1 AND deleted_at IS NULL
        "#,
        req.username,
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    verify_password(&req.password, &user.password_hash)?;

    session
        .insert(USER_ID_KEY, user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(user))
}

fn hash_password(password: &str) -> Result<String, StatusCode> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn verify_password(password: &str, hash: &str) -> Result<(), StatusCode> {
    let parsed = PasswordHash::new(hash).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| StatusCode::UNAUTHORIZED)
}
