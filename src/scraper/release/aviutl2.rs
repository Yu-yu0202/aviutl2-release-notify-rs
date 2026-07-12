use crate::util::{http, zip};
use anyhow::*;
use chrono::{DateTime, Utc};
use fancy_regex::Regex;
use regex::escape;
use reqwest::Client;
use scraper::{Html, Selector};
use tempfile::tempdir;

pub const AVIUTL2_RELEASE_URL: &str = "https://spring-fragrance.mints.ne.jp/aviutl/";

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct AviUtl2ReleaseInfo {
    pub version: String,
    pub date: DateTime<Utc>,
    pub exe_url: String,
    pub zip_url: String,
}

impl AviUtl2ReleaseInfo {
    pub fn parse(document: &Html) -> Result<(String, String, String)> {
        let information_section_selector =
            Selector::parse("html > body > center > table:nth-of-type(5)")
                .map_err(|e| anyhow!("Failed to parse selector: {}", e))?;
        let information_section = document
            .select(&information_section_selector)
            .next()
            .ok_or_else(|| anyhow!("Failed to find information section"))?;

        let information_table_selector = Selector::parse("tbody > tr > td:nth-of-type(2) > table")
            .map_err(|e| anyhow!("Failed to parse selector: {}", e))?;
        let information_table = information_section
            .select(&information_table_selector)
            .nth(0)
            .ok_or_else(|| anyhow!("Failed to find information table"))?;

        let information_selector = Selector::parse("tbody > tr > td > b > font")
            .map_err(|e| anyhow!("Failed to parse selector: {}", e))?;

        let information = information_table
            .select(&information_selector)
            .collect::<Vec<_>>();

        let mut version = information
            .get(1)
            .ok_or_else(|| anyhow!("Failed to find version"))?
            .text()
            .collect::<String>();

        version = version.trim().to_string();
        version = version.replace("を公開", "");
        version = version.replace("AviUtl ExEdit2 ", "");
        version = version.replace("version", "v");

        let version = version.trim().to_string();

        let download_table_selector = Selector::parse(
            r#"a[href^="aviutl2" i]:is([href$=".zip"], [href$=".exe"]):not([href*="sdk" i])"#,
        )
        .map_err(|e| anyhow!("Failed to parse selector: {}", e))?;
        let download_table = document
            .select(&download_table_selector)
            .collect::<Vec<_>>();

        let exe_path = download_table
            .first()
            .ok_or_else(|| anyhow!("Failed to find exe download link"))?
            .value()
            .attr("href")
            .ok_or_else(|| anyhow!("Failed to get exe download link"))?
            .to_string();

        let zip_path = download_table
            .get(1)
            .ok_or_else(|| anyhow!("Failed to find zip download link"))?
            .value()
            .attr("href")
            .ok_or_else(|| anyhow!("Failed to get zip download link"))?
            .to_string();

        let exe_url = if !exe_path.starts_with(AVIUTL2_RELEASE_URL) {
            format!("{}{}", AVIUTL2_RELEASE_URL, exe_path)
        } else {
            exe_path
        }
        .trim()
        .to_string();

        let zip_url = if !zip_path.starts_with(AVIUTL2_RELEASE_URL) {
            format!("{}{}", AVIUTL2_RELEASE_URL, zip_path)
        } else {
            zip_path
        }
        .trim()
        .to_string();

        Ok((version, exe_url, zip_url))
    }

    pub async fn from_parsed(
        version: String,
        exe_url: String,
        zip_url: String,
        client: &Client,
    ) -> Result<Self> {
        let date = crate::scraper::release::fetch_last_modified(&exe_url, client).await?;

        Ok(Self {
            version,
            date,
            exe_url,
            zip_url,
        })
    }

    pub fn is_beta(&self) -> bool {
        self.version.contains("beta")
    }

    pub fn version_number(&self) -> String {
        if self.is_beta() {
            self.version.replace("beta", "").trim().to_string()
        } else {
            self.version.replace("v", "").trim().to_string()
        }
    }
}

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct AviUtl2ReleaseChangeLog {
    pub aviutl2_changelog: Option<String>,
    pub lua_changelog: Option<String>,
}

impl AviUtl2ReleaseChangeLog {
    pub async fn fetch_by_release_info(
        release_info: &AviUtl2ReleaseInfo,
        client: &Client,
    ) -> Result<Self> {
        let tmp_dir = tempdir()?;
        let zip_target = tmp_dir
            .path()
            .join(format!("aviutl2_{}.zip", release_info.version));

        http::download_file(&release_info.zip_url, &zip_target, client).await?;

        let (aviutl2_readme, lua_readme) = tokio::join!(
            zip::read_txt_from_zip(&zip_target, "aviutl2.txt"),
            zip::read_txt_from_zip(&zip_target, "lua.txt"),
        );
        let aviutl2_readme = aviutl2_readme?;
        let lua_readme = lua_readme?;

        let aviutl2_changelog = Self::get_aviutl2_release_note(release_info, aviutl2_readme)?;
        let lua_changelog = Self::get_lua_release_note(release_info, lua_readme)?;

        Ok(Self {
            aviutl2_changelog,
            lua_changelog,
        })
    }

    fn get_aviutl2_release_note(
        release_info: &AviUtl2ReleaseInfo,
        readme: String,
    ) -> Result<Option<String>> {
        let readme = readme.replace("\r\n", "\n");
        let date_jst = release_info.date.with_timezone(&chrono_tz::Asia::Tokyo);

        let version = if release_info.is_beta() {
            format!("ver 2.00 beta{}", release_info.version_number())
        } else {
            format!("ver {}", release_info.version_number())
        };
        let date = date_jst.format("%Y/%-m/%-d").to_string();

        let re = Regex::new(&format!(
            r"(?ms)(^\s*\[{}\] {}[^\n]*\n.*?)(?=^\[\d{{4}}/\d{{1,2}}/\d{{1,2}}\]|\z)",
            escape(&date),
            escape(&version),
        ))?;

        let changelog = re.find(&readme)?.map(|m| {
            m.as_str()
                .lines()
                .map(|l| l.trim())
                .collect::<Vec<_>>()
                .join("\n")
        });

        Ok(changelog)
    }

    fn get_lua_release_note(
        release_info: &AviUtl2ReleaseInfo,
        readme: String,
    ) -> Result<Option<String>> {
        let readme = readme.replace("\r\n", "\n");
        let date_jst = release_info.date.with_timezone(&chrono_tz::Asia::Tokyo);

        let date = date_jst.format("%Y/%-m/%-d").to_string();
        let re = Regex::new(&format!(
            r"(?ms)^\s*\[{}\][^\n]*\n.*?(?=^\[\d{{4}}/\d{{1,2}}/\d{{1,2}}\]|\z)",
            escape(&date)
        ))?;

        let changelog = re
            .find_iter(&readme)
            .collect::<Result<Vec<_>, _>>()?
            .last()
            .map(|m| {
                m.as_str()
                    .lines()
                    .map(|l| l.trim())
                    .collect::<Vec<_>>()
                    .join("\n")
            });

        Ok(changelog)
    }
}
