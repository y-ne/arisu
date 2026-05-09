use serde::Serialize;
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct Camera {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_serializing)]
    pub uri: String,
    pub latitude: f64,
    pub longitude: f64,
    pub is_active: bool,
    pub created_by: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub deleted_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct CameraPublic {
    pub id: Uuid,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
}
