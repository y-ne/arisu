use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::PgPool;
use tower_sessions::Session;
use uuid::Uuid;

use crate::error::AppError;
use crate::handlers::require_admin_or_moderator;
use crate::models::camera::{Camera, CameraPublic};

#[derive(Deserialize)]
pub struct CreateCameraRequest {
    pub name: String,
    pub uri: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Deserialize)]
pub struct UpdateCameraRequest {
    pub name: Option<String>,
    pub uri: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub is_active: Option<bool>,
}

pub async fn list(State(pool): State<PgPool>) -> Result<Json<Vec<CameraPublic>>, AppError> {
    let cameras = sqlx::query_as!(
        CameraPublic,
        r#"
        SELECT id, name, latitude, longitude
        FROM cameras
        WHERE deleted_at IS NULL AND is_active = TRUE
        ORDER BY name
        "#,
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(cameras))
}

pub async fn create(
    State(pool): State<PgPool>,
    session: Session,
    Json(req): Json<CreateCameraRequest>,
) -> Result<(StatusCode, Json<Camera>), AppError> {
    let user_id = require_admin_or_moderator(&session, &pool).await?;

    if req.name.trim().is_empty() || req.uri.trim().is_empty() {
        return Err(AppError::BadRequest);
    }

    let id = Uuid::now_v7();

    let camera = sqlx::query_as!(
        Camera,
        r#"
        INSERT INTO cameras (id, name, uri, latitude, longitude, created_by)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING
            id, name, uri, latitude, longitude, is_active,
            created_by, created_at, updated_at, deleted_at
        "#,
        id,
        req.name,
        req.uri,
        req.latitude,
        req.longitude,
        user_id,
    )
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(camera)))
}

pub async fn update(
    State(pool): State<PgPool>,
    session: Session,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCameraRequest>,
) -> Result<Json<Camera>, AppError> {
    require_admin_or_moderator(&session, &pool).await?;

    let camera = sqlx::query_as!(
        Camera,
        r#"
        UPDATE cameras
        SET
            name = COALESCE($2, name),
            uri = COALESCE($3, uri),
            latitude = COALESCE($4, latitude),
            longitude = COALESCE($5, longitude),
            is_active = COALESCE($6, is_active),
            updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING
            id, name, uri, latitude, longitude, is_active,
            created_by, created_at, updated_at, deleted_at
        "#,
        id,
        req.name,
        req.uri,
        req.latitude,
        req.longitude,
        req.is_active,
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(camera))
}

pub async fn delete(
    State(pool): State<PgPool>,
    session: Session,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    require_admin_or_moderator(&session, &pool).await?;

    let result = sqlx::query!(
        "UPDATE cameras SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        id,
    )
    .execute(&pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
