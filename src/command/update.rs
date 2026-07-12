use crate::update_checker::UpdateChecker;
use crate::util::time::get_utc;
use anyhow::*;
use quicord_rs::builder::response::ResponseBuilder;
use quicord_rs::core::interaction::EmbedBuilder;
use quicord_rs::{InteractionContext, macros::slash_command};

#[slash_command(name = "update", description = "最新のリリース情報を取得します", scope = global)]
async fn update(ctx: InteractionContext) -> Result<()> {
    let embed = EmbedBuilder::new()
        .title("✅️ 進行中...")
        .description("最新の情報を取得するよう通知しました。")
        .color(0x00FF00)
        .timestamp(get_utc().to_rfc3339().parse()?)
        .build();
    let res = ResponseBuilder::new().embed(embed);

    ctx.reply(res).await?;

    UpdateChecker::check().await?;

    let embed = EmbedBuilder::new()
        .title("✅️ 完了")
        .description("最新の情報を取得しました。")
        .color(0x00FF00)
        .timestamp(get_utc().to_rfc3339().parse()?)
        .build();
    let res = ResponseBuilder::new().embed(embed);

    ctx.edit_reply(res).await?;

    Ok(())
}
