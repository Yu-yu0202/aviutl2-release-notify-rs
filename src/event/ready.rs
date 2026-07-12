use crate::notification_service::NotificationService;
use crate::update_checker::UpdateChecker;
use anyhow::*;
use quicord_rs::{EventContext, macros::event};

#[event(event = "ready", once = true)]
async fn ready(ctx: EventContext) -> Result<()> {
    NotificationService::init(
        ctx.client.http.clone(),
        ctx.storage::<crate::AppState>()?.db.clone(),
    );
    UpdateChecker::init(
        ctx.storage::<crate::AppState>()?.db.clone(),
        ctx.storage::<crate::AppState>()?.http_client.clone(),
    )
    .await?;
    UpdateChecker::check().await?;
    Ok(())
}
