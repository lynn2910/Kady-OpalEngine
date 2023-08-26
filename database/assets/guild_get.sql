SELECT
    guilds.*,
    gcg.enabled AS ghostping_enabled,
    gcg.channel AS ghostping_channel,
    gcj.enabled AS join_enabled,
    gcj.channel AS join_channel,
    gcj.message AS join_message,
    gcl.enabled AS leave_enabled,
    gcl.channel AS leave_channel,
    gcl.message AS leave_message,
    g.enabled AS logs_enabled,
    gcs.enabled AS suggestions_enabled,
    gcs.channel AS suggestions_channel,
    x.enabled AS xp_enabled,
    x.cooldown AS xp_cooldown,
    x.algorithm as xp_algo,
    x.channel as xp_channel,
    x.message as xp_message,
    gc.enabled AS captcha_enabled,
    gc.channel AS captcha_channel,
    gc.role AS captcha_role,
    gc.model AS captcha_model,
    gc.level AS captcha_level,
    ar.enabled as auto_role_enabled,
    gcci.enabled as citation_enabled,
    gcci.channel as citation_channel
FROM guilds
    LEFT OUTER JOIN guild_config_ghostping gcg on guilds.id = gcg.guild_id
    LEFT OUTER JOIN guild_config_join gcj on guilds.id = gcj.guild_id
    LEFT OUTER JOIN guild_config_leave gcl on guilds.id = gcl.guild_id
    LEFT OUTER JOIN guild_config_logs g on guilds.id = g.guild_id
    LEFT OUTER JOIN guild_config_suggestions gcs on guilds.id = gcs.guild_id
    LEFT OUTER JOIN guild_config_xp x on guilds.id = x.guild_id
    LEFT OUTER JOIN guild_config_captcha gc on guilds.id = gc.guild_id
    LEFT OUTER JOIN guild_config_auto_roles ar on guilds.id = ar.guild_id
    LEFT OUTER JOIN guild_config_citation gcci on guilds.id = gcci.guild_id
WHERE id = ?;