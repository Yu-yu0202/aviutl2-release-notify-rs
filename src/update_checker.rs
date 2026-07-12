use crate::notification_service::NotificationService;
use crate::scraper::release::Release;
use anyhow::*;
use quicord_rs::log::info;
use reqwest::Client;
use sqlx::PgPool;
use std::sync::{Arc, OnceLock};
use tokio_cron_scheduler::{Job, JobScheduler};

static UPDATE_CHECKER: OnceLock<UpdateChecker> = OnceLock::new();

#[derive(Clone)]
pub struct UpdateChecker {
    db: Arc<PgPool>,
    client: Arc<Client>,
    #[allow(dead_code)]
    schedule: JobScheduler,
}

impl UpdateChecker {
    pub async fn init(db: Arc<PgPool>, client: Arc<Client>) -> Result<()> {
        let schedule = JobScheduler::new().await?;
        schedule
            .add(Job::new_async("0 0 */3 * * *", move |_uuid, _l| {
                Box::pin(async move {
                    if let Err(e) = UpdateChecker::check().await {
                        eprintln!("Error during update check: {:?}", e);
                    }
                })
            })?)
            .await?;
        schedule.start().await?;

        UPDATE_CHECKER
            .set(Self {
                db,
                client,
                schedule,
            })
            .unwrap_or_else(|_| panic!("UpdateChecker already initialized"));

        Ok(())
    }

    pub async fn check() -> Result<()> {
        let checker = UPDATE_CHECKER
            .get()
            .ok_or_else(|| anyhow!("UpdateChecker is not initialized"))?;

        checker.run().await
    }

    async fn run(&self) -> Result<()> {
        info!("Checking for updates...");

        if let Some(ref release) = Release::check_update(self.db.clone(), &self.client).await? {
            if let Some(ref aviutl2) = release.aviutl2 {
                info!("New AviUtl2 release found: {}", aviutl2.version);
            }

            if release.sdk.is_some() {
                info!("New SDK release found.");
            }

            NotificationService::notify(release).await?;
            release.push_to_db(self.db.clone()).await?;
        } else {
            info!("No new releases found.");
        }

        info!("Update check completed.");

        Ok(())
    }
}
