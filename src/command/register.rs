use crate::model::registration::Registration;
use crate::util::permission::has_permission_in_guild;
use crate::util::time::get_utc;
use anyhow::*;
use quicord_rs::builder::response::ResponseBuilder;
use quicord_rs::core::interaction::EmbedBuilder;
use quicord_rs::{InteractionContext, macros::slash_command};

#[slash_command(name = "register", description = "通知の送信先を登録します", scope = global, options = [
    Channel("対象チャンネル")
])]
async fn register(ctx: InteractionContext) -> Result<()> {
    if ctx.guild_id().is_none() {
        let embed = EmbedBuilder::new()
            .title("❌️ エラー")
            .description("このコマンドはサーバー内でのみ使用できます")
            .build();

        let res = ResponseBuilder::new().embed(embed);

        ctx.reply(res).await?;

        return Ok(());
    };

    if ctx.author_id().is_none() {
        let embed = EmbedBuilder::new()
            .title("❌️ エラー")
            .description("予期せぬエラーが発生しました")
            .build();

        let res = ResponseBuilder::new().embed(embed);

        ctx.reply(res).await?;

        return Ok(());
    };

    if !has_permission_in_guild(
        &ctx.client,
        ctx.guild_id().unwrap(),
        ctx.author_id().unwrap(),
        twilight_model::guild::Permissions::MANAGE_CHANNELS,
    )
    .await?
    {
        let embed = EmbedBuilder::new()
            .title("❌️ エラー")
            .description("このコマンドを実行するには `サーバーを管理` 権限が必要です。")
            .build();

        let res = ResponseBuilder::new().embed(embed);

        ctx.reply(res).await?;

        return Ok(());
    }

    ctx.defer_reply(false).await?;

    let state = ctx.storage::<crate::AppState>()?;
    let options = ctx.options().unwrap();
    let target_channel = options.channel("対象チャンネル").unwrap();

    let registration = Registration {
        channel_id: target_channel.to_string(),
    };

    Registration::create(&state.db, &registration).await?;

    let embed = EmbedBuilder::new()
        .title("✅️ 完了")
        .description(format!("<#{}> に通知を設定しました。", target_channel))
        .timestamp(get_utc().to_rfc3339().parse()?)
        .color(0x00FF00)
        .build();

    let res = ResponseBuilder::new().embed(embed);

    ctx.edit_reply(res).await?;

    Ok(())
}
