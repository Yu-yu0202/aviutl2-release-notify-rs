use crate::scraper::release::aviutl2::AVIUTL2_RELEASE_URL;
use crate::util::{http, zip};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use fancy_regex::Regex;
use regex::escape;
use reqwest::Client;
use scraper::{Html, Selector};
use tempfile::tempdir;

pub const AVIUTL2_SDK_URL: &str = "https://spring-fragrance.mints.ne.jp/aviutl/aviutl2_sdk.zip";

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct SdkReleaseInfo {
    pub date: DateTime<Utc>,
}

impl SdkReleaseInfo {
    pub fn parse(document: &Html) -> anyhow::Result<String> {
        let link_selector = Selector::parse(r#"a[href$="aviutl2_sdk.zip"]"#)
            .map_err(|e| anyhow!("Failed to parse selector: {}", e))?;

        document
            .select(&link_selector)
            .next()
            .and_then(|e| e.value().attr("href"))
            .map(|s| {
                if s.starts_with(AVIUTL2_RELEASE_URL) {
                    s.to_string()
                } else {
                    format!("{}{}", AVIUTL2_RELEASE_URL, s)
                }
            })
            .ok_or_else(|| anyhow!("Failed to find SDK download link"))
    }

    pub async fn from_parsed(link: String, client: &Client) -> anyhow::Result<Self> {
        let date = crate::scraper::release::fetch_last_modified(&link, client).await?;

        anyhow::Ok(Self { date })
    }
}

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct SdkReleaseChangeLog {
    pub changelog: Option<String>,
}

impl SdkReleaseChangeLog {
    pub async fn fetch_by_release_info(
        release_info: &SdkReleaseInfo,
        client: &Client,
    ) -> anyhow::Result<Self> {
        let date_jst = release_info.date.with_timezone(&chrono_tz::Asia::Tokyo);

        let tmp_dir = tempdir()?;
        let zip_target = tmp_dir.path().join("aviutl2_sdk.zip");

        http::download_file(AVIUTL2_SDK_URL, &zip_target, client).await?;

        let readme = zip::read_txt_from_zip(&zip_target, "aviutl2_plugin_sdk.txt").await?;
        let readme = readme.replace("\r\n", "\n");

        let date = date_jst.format("%Y/%-m/%-d").to_string();
        let re = Regex::new(&format!(
            r"(?ms)^\s*\[{}\][^\n]*\n.*?(?=^\[\d{{4}}/\d{{1,2}}/\d{{1,2}}\]|\z)",
            escape(&date)
        ))?;

        let changelog = re
            .find_iter(&readme)
            .collect::<anyhow::Result<Vec<_>, _>>()?
            .last()
            .map(|m| {
                m.as_str()
                    .lines()
                    .map(|l| l.trim())
                    .collect::<Vec<_>>()
                    .join("\n")
            });

        anyhow::Ok(Self { changelog })
    }
}
