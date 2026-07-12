use crate::model::aviutl2_release::AviUtl2Release;
use crate::model::sdk_release::SdkRelease;
use crate::scraper::release::aviutl2::{
    AVIUTL2_RELEASE_URL, AviUtl2ReleaseChangeLog, AviUtl2ReleaseInfo,
};
use crate::scraper::release::sdk::{SdkReleaseChangeLog, SdkReleaseInfo};
use crate::util::http;
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use reqwest::Client;
use scraper::Html;
use sqlx::PgPool;
use std::sync::Arc;

pub mod aviutl2;
pub mod sdk;

pub async fn fetch_last_modified(
    url: &str,
    client: &Client,
) -> anyhow::Result<DateTime<Utc>> {
    let head_res = client.head(url).send().await?;

    if !head_res.status().is_success() {
        return Err(anyhow!("Failed to fetch download link: {}", url));
    }

    let last_modified = head_res
        .headers()
        .get("last-modified")
        .ok_or_else(|| anyhow!("Failed to get last-modified header"))?
        .to_str()
        .map_err(|e| anyhow!("Failed to convert last-modified header to str: {}", e))?;

    anyhow::Ok(DateTime::parse_from_rfc2822(last_modified)?.to_utc())
}

async fn fetch_latest_release_info(
    client: &Client,
) -> anyhow::Result<(AviUtl2ReleaseInfo, SdkReleaseInfo)> {
    let res = http::get_url_content(AVIUTL2_RELEASE_URL, client).await?;
    let (aviutl2_parsed, sdk_link) = {
        let document = Html::parse_document(&res);
        (
            AviUtl2ReleaseInfo::parse(&document)?,
            SdkReleaseInfo::parse(&document)?,
        )
    };

    let (version, exe_url, zip_url) = aviutl2_parsed;

    let (aviutl2_release_info, sdk_release) = tokio::join!(
        AviUtl2ReleaseInfo::from_parsed(version, exe_url, zip_url, client),
        SdkReleaseInfo::from_parsed(sdk_link, client),
    );

    anyhow::Ok((aviutl2_release_info?, sdk_release?))
}

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct Release {
    pub aviutl2: Option<AviUtl2Release>,
    pub sdk: Option<SdkRelease>,
}

impl Release {
    pub async fn check_update(db: Arc<PgPool>, client: &Client) -> anyhow::Result<Option<Self>> {
        let current_release_info = fetch_latest_release_info(client).await?;
        let (current_aviutl2_release_info, current_sdk_release_info) = current_release_info;

        let (last_aviutl2_release, last_sdk_release) =
            tokio::join!(AviUtl2Release::get_latest(&db), SdkRelease::get_latest(&db),);
        let last_aviutl2_release = last_aviutl2_release?;
        let last_sdk_release = last_sdk_release?;

        // aviutl2, sdk
        let mut updated = (false, false);

        if let Some(last_aviutl2_release) = last_aviutl2_release {
            let aviutl2_release = AviUtl2Release::from(
                &current_aviutl2_release_info,
                &AviUtl2ReleaseChangeLog::default(),
            );

            if last_aviutl2_release < aviutl2_release {
                updated = (true, updated.1);
            }
        } else {
            updated = (true, updated.1);
        }

        if let Some(last_sdk_release) = last_sdk_release {
            let sdk_release =
                SdkRelease::from(&current_sdk_release_info, &SdkReleaseChangeLog::default());

            if last_sdk_release < sdk_release {
                updated = (updated.0, true);
            }
        } else {
            updated = (updated.0, true);
        }

        if updated == (false, false) {
            return anyhow::Ok(None);
        }

        let (aviutl2_release, sdk_release);

        if updated.0 {
            let aviutl2_changelog = AviUtl2ReleaseChangeLog::fetch_by_release_info(
                &current_aviutl2_release_info,
                client,
            )
            .await?;
            aviutl2_release = Some(AviUtl2Release::from(
                &current_aviutl2_release_info,
                &aviutl2_changelog,
            ));
        } else {
            aviutl2_release = None;
        }

        if updated.1 {
            let sdk_changelog =
                SdkReleaseChangeLog::fetch_by_release_info(&current_sdk_release_info, client)
                    .await?;
            sdk_release = Some(SdkRelease::from(&current_sdk_release_info, &sdk_changelog));
        } else {
            sdk_release = None;
        }

        anyhow::Ok(Some(Release {
            aviutl2: aviutl2_release,
            sdk: sdk_release,
        }))
    }

    pub async fn push_to_db(&self, db: Arc<PgPool>) -> anyhow::Result<()> {
        if let Some(aviutl2_release) = &self.aviutl2 {
            AviUtl2Release::create(&db, aviutl2_release).await?;
        }
        if let Some(sdk_release) = &self.sdk {
            SdkRelease::create(&db, sdk_release).await?;
        }
        anyhow::Ok(())
    }
}
