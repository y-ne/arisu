use sqlx::PgPool;
use tower_sessions::Session;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::user::UserRole;

pub mod auth;
pub mod cameras;
pub mod root;

pub const USER_ID_KEY: &str = "user_id";

pub async fn require_auth(session: &Session) -> Result<Uuid, AppError> {
    session
        .get::<Uuid>(USER_ID_KEY)
        .await?
        .ok_or(AppError::Unauthorized)
}

pub async fn require_admin_or_moderator(
    session: &Session,
    pool: &PgPool,
) -> Result<Uuid, AppError> {
    let user_id = require_auth(session).await?;

    let role = sqlx::query_scalar!(
        r#"SELECT role AS "role: UserRole" FROM users WHERE id = $1 AND deleted_at IS NULL"#,
        user_id,
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    match role {
        UserRole::Administrator | UserRole::Moderator => Ok(user_id),
        _ => Err(AppError::Forbidden),
    }
}
