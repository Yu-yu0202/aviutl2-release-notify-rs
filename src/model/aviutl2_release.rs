use crate::scraper::release::aviutl2::{AviUtl2ReleaseChangeLog, AviUtl2ReleaseInfo};
use anyhow::*;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(FromRow, Eq, PartialEq, Clone, Debug)]
pub struct AviUtl2Release {
    pub id: Uuid,
    pub version: String,
    pub zip_url: String,
    pub exe_url: String,
    pub release_date: DateTime<Utc>,
    pub aviutl2_changelog: Option<String>,
    pub lua_changelog: Option<String>,
}

impl PartialOrd<Self> for AviUtl2Release {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AviUtl2Release {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.release_date.cmp(&other.release_date)
    }
}

impl AviUtl2Release {
    pub fn from(release_info: &AviUtl2ReleaseInfo, change_log: &AviUtl2ReleaseChangeLog) -> Self {
        Self {
            id: Uuid::now_v7(),
            version: release_info.version.clone(),
            zip_url: release_info.zip_url.clone(),
            exe_url: release_info.exe_url.clone(),
            release_date: release_info.date,
            aviutl2_changelog: change_log.aviutl2_changelog.clone(),
            lua_changelog: change_log.lua_changelog.clone(),
        }
    }

    pub async fn create(pool: &PgPool, release: &AviUtl2Release) -> Result<()> {
        let uuid = Uuid::now_v7();

        sqlx::query!(
            r#"
            INSERT INTO aviutl2_releases (id, version, zip_url, exe_url, release_date, aviutl2_changelog, lua_changelog)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            &uuid,
            &release.version,
            &release.zip_url,
            &release.exe_url,
            &release.release_date,
            release.aviutl2_changelog.as_deref(),
            release.lua_changelog.as_deref()
        )
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_latest(pool: &PgPool) -> Result<Option<AviUtl2Release>> {
        let release = sqlx::query_as!(
            AviUtl2Release,
            r#"
            SELECT * FROM aviutl2_releases ORDER BY release_date DESC LIMIT 1
            "#
        )
        .fetch_optional(pool)
        .await?;

        Ok(release)
    }

    pub async fn get_all(pool: &PgPool) -> Result<Vec<AviUtl2Release>> {
        let releases = sqlx::query_as!(
            AviUtl2Release,
            r#"
            SELECT * FROM aviutl2_releases ORDER BY release_date DESC
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(releases)
    }

    pub async fn get_by_version(pool: &PgPool, version: &str) -> Result<Option<AviUtl2Release>> {
        let release = sqlx::query_as!(
            AviUtl2Release,
            r#"
            SELECT * FROM aviutl2_releases WHERE version = $1 ORDER BY release_date DESC LIMIT 1
            "#,
            version
        )
        .fetch_optional(pool)
        .await?;

        Ok(release)
    }

    pub async fn delete_by_version(pool: &PgPool, version: &str) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM aviutl2_releases WHERE version = $1
            "#,
            version
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete_by_id(pool: &PgPool, id: &Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM aviutl2_releases WHERE id = $1
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
            DELETE FROM aviutl2_releases
            "#
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
