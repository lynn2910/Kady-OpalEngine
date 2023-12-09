use client::models::user::UserId;
use config::Config;

pub const DEFAULT_LANG: &str = "fr";
pub const DEFAULT_AVATAR: &str = "https://cdn.discordapp.com/embed/avatars/0.png";

pub(crate) const ADMIN_GUILD: &str = "1135937845635317861";
pub(crate) const ADMINS_ACTIVITY_REPORT: &str = "1135982161888026634";
pub(crate) const ADMINS: &[&[u8]] = &[
    &[131, 31, 245, 87, 151, 67, 124, 90, 223, 238, 1, 245, 99, 115, 67, 58, 24, 91, 98, 113, 222, 162, 184, 40, 179, 255, 10, 154, 33, 96, 156, 189],
    &[185, 211, 140, 235, 41, 30, 201, 172, 20, 187, 247, 105, 165, 1, 246, 188, 105, 1, 55, 181, 103, 99, 157, 179, 224, 208, 166, 154, 162, 133, 111, 253],
    &[113, 1, 197, 164, 66, 212, 109, 247, 87, 186, 105, 168, 50, 61, 115, 26, 171, 83, 246, 15, 117, 252, 254, 15, 134, 252, 200, 219, 95, 58, 246, 26]
];

pub(crate) fn generate_invite_link(client_id: &UserId, config: &Config) -> String {
    format!(
        "https://discord.com/oauth2/authorize?client_id={id}&permissions={perms}&scope={scope}",
        id = client_id.0,
        perms = config.client.invite_required_permissions,
        scope = config.client.invite_scope
    )
}