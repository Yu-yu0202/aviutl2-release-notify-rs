use anyhow::*;
use sqlx::{FromRow, PgPool};

#[derive(FromRow, Eq, PartialEq, Clone, Debug)]
pub struct Registration {
    pub channel_id: String,
}

impl Registration {
    pub async fn create(pool: &PgPool, registration: &Registration) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO registrations (channel_id)
            VALUES ($1)
            ON CONFLICT DO NOTHING
            "#,
            &registration.channel_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_all(pool: &PgPool) -> Result<Vec<Registration>> {
        let registrations = sqlx::query_as!(
            Registration,
            r#"
            SELECT * FROM registrations
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(registrations)
    }

    pub async fn get_by_id(pool: &PgPool, id: &str) -> Result<Option<Registration>> {
        let registration = sqlx::query_as!(
            Registration,
            r#"
            SELECT * FROM registrations WHERE channel_id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(registration)
    }

    pub async fn exists(pool: &PgPool, id: &str) -> Result<bool> {
        let count = sqlx::query!(
            r#"
            SELECT COUNT(*) FROM registrations WHERE channel_id = $1
            "#,
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(count.count.unwrap_or(0) > 0)
    }

    pub async fn delete_by_id(pool: &PgPool, id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM registrations WHERE channel_id = $1
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete_all(pool: &PgPool) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM registrations
            "#
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
