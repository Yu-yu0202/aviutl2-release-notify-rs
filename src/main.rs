use anyhow::*;
use quicord_rs::BotBuilder;
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::{Arc, OnceLock};

mod command;
mod event;
mod model;
mod notification_service;
mod scraper;
mod update_checker;
mod util;

#[derive(Deserialize, Clone, Default, Debug)]
pub struct Config {
    pub discord_token: String,
    pub database_url: String,
}

pub static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn config() -> &'static Config {
    CONFIG.get().expect("Failed to read environment variables")
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: Arc<PgPool>,
    pub http_client: Arc<reqwest::Client>,
}

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    dotenvy::dotenv().ok();

    CONFIG
        .set(envy::from_env::<Config>()?)
        .expect("Failed to read environment variables");

    let db = Arc::new(PgPool::connect(&config().database_url).await?);
    let http_client = Arc::new(reqwest::Client::new());
    let state = AppState { db, http_client };

    sqlx::migrate!("./migrations").run(&*state.db).await?;

    let bot = BotBuilder::new(config().discord_token.clone())
        .storage()
        .insert(state)
        .build()
        .await?;

    bot.start().await?;

    Ok(())
}
