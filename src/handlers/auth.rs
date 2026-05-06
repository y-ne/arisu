use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::User;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub display_name: String,
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

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    let id = Uuid::now_v7();

    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (id, username, display_name, password_hash)
        VALUES ($1, $2, $3, $4)
        RETURNING
            id,
            username,
            display_name,
            password_hash,
            role AS "role: _",
            created_at,
            updated_at,
            deleted_at
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
