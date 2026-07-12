use crate::scraper::release::sdk::{SdkReleaseChangeLog, SdkReleaseInfo};
use anyhow::*;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use std::cmp::Ordering;
use uuid::Uuid;

#[derive(FromRow, Eq, PartialEq, Clone, Debug)]
pub struct SdkRelease {
    pub id: Uuid,
    pub release_date: DateTime<Utc>,
    pub changelog: Option<String>,
}

impl PartialOrd<Self> for SdkRelease {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SdkRelease {
    fn cmp(&self, other: &Self) -> Ordering {
        self.release_date.cmp(&other.release_date)
    }
}

impl SdkRelease {
    pub fn from(release_info: &SdkReleaseInfo, changelog: &SdkReleaseChangeLog) -> Self {
        Self {
            id: Uuid::now_v7(),
            release_date: release_info.date,
            changelog: changelog.changelog.clone(),
        }
    }

    pub async fn create(pool: &PgPool, release: &SdkRelease) -> Result<()> {
        let uuid = Uuid::now_v7();
        sqlx::query!(
            r#"
            INSERT INTO sdk_releases (id, release_date, changelog)
            VALUES ($1, $2, $3)
            "#,
            &uuid,
            &release.release_date,
            release.changelog.as_deref()
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_by_date(pool: &PgPool, date: &DateTime<Utc>) -> Result<Option<SdkRelease>> {
        let release = sqlx::query_as!(
            SdkRelease,
            r#"
            SELECT * FROM sdk_releases WHERE release_date = $1 ORDER BY release_date DESC LIMIT 1
            "#,
            date
        )
        .fetch_optional(pool)
        .await?;

        Ok(release)
    }

    pub async fn get_latest(pool: &PgPool) -> Result<Option<SdkRelease>> {
        let release = sqlx::query_as!(
            SdkRelease,
            r#"
            SELECT * FROM sdk_releases ORDER BY release_date DESC LIMIT 1
            "#
        )
        .fetch_optional(pool)
        .await?;

        Ok(release)
    }

    pub async fn get_all(pool: &PgPool) -> Result<Vec<SdkRelease>> {
        let releases = sqlx::query_as!(
            SdkRelease,
            r#"
            SELECT * FROM sdk_releases ORDER BY release_date DESC
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(releases)
    }

    pub async fn delete_by_date(pool: &PgPool, date: &DateTime<Utc>) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM sdk_releases WHERE release_date = $1
            "#,
            date
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete_by_id(pool: &PgPool, id: &Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM sdk_releases WHERE id = $1
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
            DELETE FROM sdk_releases
            "#
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
