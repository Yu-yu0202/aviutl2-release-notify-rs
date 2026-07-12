use crate::model::registration::Registration;
use crate::scraper::release::Release;
use crate::scraper::release::aviutl2::AVIUTL2_RELEASE_URL;
use anyhow::*;
use quicord_rs::builder::button::ButtonBuilder;
use quicord_rs::core::interaction::{ContainerBuilder, TextDisplayBuilder};
use sqlx::PgPool;
use std::sync::{Arc, OnceLock};
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::component::{ButtonStyle, SeparatorSpacingSize};
use twilight_model::channel::message::{Component, MessageFlags};
use twilight_util::builder::message::{SectionBuilder, SeparatorBuilder};

static NOTIFICATION_SERVICE: OnceLock<NotificationService> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct NotificationService {
    http: Arc<HttpClient>,
    db: Arc<PgPool>,
}

impl NotificationService {
    pub fn init(http: Arc<HttpClient>, db: Arc<PgPool>) {
        NOTIFICATION_SERVICE
            .set(Self { http, db })
            .unwrap_or_else(|_| panic!("NotificationService already initialized"));
    }

    pub async fn notify(release: &Release) -> Result<()> {
        let service = NOTIFICATION_SERVICE
            .get()
            .ok_or_else(|| anyhow!("NotificationService is not initialized"))?;

        service.send(release).await
    }

    async fn send(&self, release: &Release) -> Result<()> {
        let channels = Registration::get_all(&self.db).await?;
        for channel in channels {
            if let Some(container) = create_aviutl2_container(release) {
                self.http
                    .create_message(channel.channel_id.parse()?)
                    .flags(MessageFlags::IS_COMPONENTS_V2)
                    .components(&[container])
                    .await?;
            }
            if let Some(container) = create_sdk_container(release) {
                self.http
                    .create_message(channel.channel_id.parse()?)
                    .flags(MessageFlags::IS_COMPONENTS_V2)
                    .components(&[container])
                    .await?;
            }
        }

        Ok(())
    }
}

pub fn create_aviutl2_container(release: &Release) -> Option<Component> {
    release.aviutl2
        .as_ref()
        .map(|aviutl2_release|
            ContainerBuilder::new()
                .component(
                    TextDisplayBuilder::new(format!("### AviUtl ExEdit2 {} が公開されました！", aviutl2_release.version))
                        .build(),
                )
                .component(
                    TextDisplayBuilder::new("更新点(aviutl2.txt から抜粋):").build()
                )
                .component(
                    TextDisplayBuilder::new(format!("```plaintext\n{}\n```", aviutl2_release.aviutl2_changelog.clone().unwrap_or("情報が見つかりません".to_string()))).build()
                )
                .component(
                    TextDisplayBuilder::new("更新点(lua.txt から抜粋):").build()
                )
                .component(
                    TextDisplayBuilder::new(format!("```plaintext\n{}\n```", aviutl2_release.lua_changelog.clone().unwrap_or("情報が見つかりません".to_string()))).build()
                )
                .component(
                    SeparatorBuilder::new().spacing(SeparatorSpacingSize::Large).build()
                )
                .component(
                    TextDisplayBuilder::new(format!("AviUtl ExEdit2 {} のダウンロードリンク", aviutl2_release.version))
                        .build()
                )
                .component(
                    SectionBuilder::new(
                        ButtonBuilder::new(ButtonStyle::Link)
                            .label("ダウンロード")
                            .url(aviutl2_release.exe_url.clone())
                    )
                        .component(
                            TextDisplayBuilder::new("インストーラー版:")
                                .build()
                        )
                        .build()
                )
                .component(
                    SectionBuilder::new(
                        ButtonBuilder::new(ButtonStyle::Link)
                            .label("ダウンロード")
                            .url(aviutl2_release.zip_url.clone())
                    )
                        .component(
                            TextDisplayBuilder::new("ZIP版:")
                                .build()
                        )
                        .build()
                )
                .component(
                    SectionBuilder::new(
                        ButtonBuilder::new(ButtonStyle::Link)
                            .label("開く")
                            .url(AVIUTL2_RELEASE_URL)
                    )
                        .component(
                            TextDisplayBuilder::new("AviUtlのお部屋:")
                                .build()
                        )
                        .build()
                )
                .component(
                    SeparatorBuilder::new().spacing(SeparatorSpacingSize::Small).build()
                )
                .component(
                    TextDisplayBuilder::new(
                        "-# このツールの使用により生じた問題は、作者は一切責任を負いません。\n-# 正確な情報を確認してください。"
                    ).build()
                )
                .component(
                    TextDisplayBuilder::new(
                        "-# [AviUtl2 Release Notify](<https://discord.com/oauth2/authorize?client_id=1416628772710645851>) by Yu-yu0202"
                    ).build()
                )
                .build()
                .into()
        )
}

pub fn create_sdk_container(release: &Release) -> Option<Component> {
    release.sdk
        .as_ref()
        .map(|sdk_release|
            ContainerBuilder::new()
                .component(
                    TextDisplayBuilder::new("### AviUtl ExEdit2 Plugin SDK が更新されました！")
                        .build(),
                )
                .component(
                    TextDisplayBuilder::new("更新点(aviutl2_plugin_sdk.txtから抜粋):").build(),
                )
                .component(
                    TextDisplayBuilder::new(format!(
                        "```plaintext\n{}\n```",
                        sdk_release
                            .changelog
                            .clone()
                            .unwrap_or("情報が見つかりません".to_string())
                    ))
                        .build(),
                )
                .component(
                    SeparatorBuilder::new()
                        .spacing(SeparatorSpacingSize::Large)
                        .build(),
                )
                .component(
                    SectionBuilder::new(
                        ButtonBuilder::new(ButtonStyle::Link)
                            .label("開く")
                            .url(AVIUTL2_RELEASE_URL),
                    )
                        .component(TextDisplayBuilder::new("AviUtlのお部屋:").build())
                        .build(),
                )
                .component(
                    SeparatorBuilder::new().spacing(SeparatorSpacingSize::Small).build()
                )
                .component(
                    TextDisplayBuilder::new(
                        "-# このツールの使用により生じた問題は、作者は一切責任を負いません。\n-# 正確な情報を確認してください。"
                    ).build()
                )
                .component(
                    TextDisplayBuilder::new(
                        "-# [AviUtl2 Release Notify](<https://discord.com/oauth2/authorize?client_id=1416628772710645851>) by Yu-yu0202"
                    ).build()
                )
                .build()
                .into()
        )
}
