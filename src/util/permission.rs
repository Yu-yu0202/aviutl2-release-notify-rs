use anyhow::*;
use quicord_rs::core::client::Client;
use twilight_model::guild::Permissions;
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, RoleMarker, UserMarker};

pub async fn has_permission_in_guild(
    client: &Client,
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    permission: Permissions,
) -> Result<bool> {
    let guild = client.http.guild(guild_id).await?.model().await?;

    if guild.owner_id == user_id {
        return Ok(true);
    }

    let member = client
        .http
        .guild_member(guild_id, user_id)
        .await?
        .model()
        .await?;

    let member_role_permission: Vec<(Id<RoleMarker>, Permissions)> = guild
        .roles
        .iter()
        .filter(|role| member.roles.contains(&role.id))
        .map(|role| (role.id, role.permissions))
        .collect();

    let everyone_permission = guild
        .roles
        .iter()
        .find(|role| role.id == guild_id.cast())
        .map(|role| role.permissions)
        .unwrap_or_else(Permissions::empty);

    let user_permissions = member_role_permission
        .into_iter()
        .fold(everyone_permission, |acc, (_, perm)| acc | perm);

    Ok(user_permissions.contains(permission)
        || user_permissions.contains(Permissions::ADMINISTRATOR))
}
