DROP TABLE IF EXISTS guild_logs;
DROP TABLE IF EXISTS guild_config_logs;
DROP TABLE IF EXISTS guild_users_xp;
DROP TABLE IF EXISTS guild_config_xp;
DROP TABLE IF EXISTS guild_system_logs;
DROP TABLE IF EXISTS guild_config_leave;
DROP TABLE IF EXISTS guild_config_join;
DROP TABLE IF EXISTS guild_config_suggestions;
DROP TABLE IF EXISTS guild_config_ghostping;
DROP TABLE IF EXISTS guild_config_captcha;
DROP TABLE IF EXISTS guild_config_auto_roles;
DROP TABLE IF EXISTS guild_auto_roles;
DROP TABLE IF EXISTS guilds;

# Contains the guild default information's
CREATE OR REPLACE TABLE guilds
(
    id                    VARCHAR(32) NOT NULL,
    tos_accepted          BOOLEAN NOT NULL DEFAULT FALSE,
    lang                  VARCHAR(2) NOT NULL DEFAULT 'fr',
    # If the bot will join every threads
    join_threads          BOOLEAN NOT NULL DEFAULT TRUE,
    # Sapphire, aka premium
    sapphire              BOOLEAN NOT NULL DEFAULT 0,

    # Used to auto-delete the data (RGPD friendly)
    last_seen             DATETIME    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_edited_timestamp DATETIME    NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (id)
);

# Contains the guild config for the xp system
CREATE OR REPLACE TABLE guild_config_xp
(
    guild_id    VARCHAR(32) NOT NULL,
    enabled     BOOLEAN NOT NULL DEFAULT FALSE,
    # The cooldown in seconds
    cooldown    INTEGER UNSIGNED DEFAULT NULL,
    algorithm   INTEGER UNSIGNED NOT NULL DEFAULT 0,
    message     VARCHAR(512) DEFAULT NULL,
    channel     VARCHAR(32) DEFAULT NULL,

    PRIMARY KEY (guild_id),
    FOREIGN KEY (guild_id) REFERENCES guilds (id) ON DELETE CASCADE,
    CHECK (cooldown >= 0)
);

# Contains each member xp
CREATE OR REPLACE TABLE guild_users_xp
(
    guild_id  VARCHAR(32)      NOT NULL,
    user_id   VARCHAR(32)      NOT NULL,
    xp        INTEGER UNSIGNED NOT NULL DEFAULT 0,

    PRIMARY KEY (guild_id, user_id),
    FOREIGN KEY (guild_id) REFERENCES guilds (id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    CHECK (xp >= 0 )
);

# Contains the guild config for the logs
CREATE OR REPLACE TABLE guild_config_logs
(
    guild_id VARCHAR(32) NOT NULL,
    enabled  BOOLEAN NOT NULL DEFAULT FALSE,

    PRIMARY KEY (guild_id),
    FOREIGN KEY (guild_id) REFERENCES guilds (id) ON DELETE CASCADE
);

# Contains the guild logs with their channels
CREATE OR REPLACE TABLE guild_logs
(
    # The guild id
    guild_id VARCHAR(32) NOT NULL,
    # The channel id
    channel  VARCHAR(32) NOT NULL,
    # The log type
    log_type VARCHAR(32) NOT NULL,

    PRIMARY KEY (guild_id, log_type),
    FOREIGN KEY (guild_id) REFERENCES guilds (id) ON DELETE CASCADE
);

# Contains the guild config for the leave messages
CREATE OR REPLACE TABLE guild_config_leave
(
    guild_id VARCHAR(32)  NOT NULL,
    enabled  BOOLEAN      NOT NULL DEFAULT FALSE,
    channel  VARCHAR(32),
    message  VARCHAR(255),

    PRIMARY KEY (guild_id),
    FOREIGN KEY (guild_id) REFERENCES guilds (id) ON DELETE CASCADE
);

# Contains the guild config for the join messages
CREATE OR REPLACE TABLE guild_config_join
(
    guild_id VARCHAR(32)  NOT NULL,
    enabled  BOOLEAN      NOT NULL DEFAULT FALSE,
    channel  VARCHAR(32),
    message  VARCHAR(255),

    PRIMARY KEY (guild_id),
    FOREIGN KEY (guild_id) REFERENCES guilds (id) ON DELETE CASCADE
);

# Contains the guild config for the suggestions
CREATE OR REPLACE TABLE guild_config_suggestions
(
    guild_id VARCHAR(32) NOT NULL,
    enabled  BOOLEAN     NOT NULL DEFAULT FALSE,
    channel  VARCHAR(32),

    PRIMARY KEY (guild_id),
    FOREIGN KEY (guild_id) REFERENCES guilds (id) ON DELETE CASCADE
);

# Contains the guild config for the ghostping system
CREATE OR REPLACE TABLE guild_config_ghostping
(
    guild_id VARCHAR(32) NOT NULL,
    enabled  BOOLEAN     NOT NULL DEFAULT FALSE,
    channel  VARCHAR(32),

    PRIMARY KEY (guild_id),
    FOREIGN KEY (guild_id) REFERENCES guilds (id) ON DELETE CASCADE
);

# Contains the guild system logs
# Will store each action done by anyone on the guild
CREATE OR REPLACE TABLE guild_system_logs
(
    # The guild id
    guild     VARCHAR(32) NOT NULL,
    # The author of the action
    author    VARCHAR(32) NOT NULL,
    # The action done
    action    INT UNSIGNED NOT NULL,
    # The target of the action
    target    VARCHAR(32) NOT NULL,
    # The reason of the action
    reason    VARCHAR(255) NOT NULL,
    # When the action was done
    # Will be also used to auto-delete the data (RGPD friendly)
    # Will be deleted after 14 days
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (guild, timestamp, action, target),
    FOREIGN KEY (guild) REFERENCES guilds (id) ON DELETE CASCADE
);

CREATE OR REPLACE TABLE guild_config_captcha (
    guild_id VARCHAR(32) NOT NULL,
    enabled  BOOLEAN     NOT NULL DEFAULT FALSE,
    channel  VARCHAR(32),
    role     VARCHAR(32),
    # The captcha model (amelia, lucy, mila)
    model    VARCHAR(32),
    # The captcha difficulty (easy = 1, medium = 2, hard = 3)
    level    INT UNSIGNED NOT NULL DEFAULT 1,

    PRIMARY KEY (guild_id),
    FOREIGN KEY (guild_id) REFERENCES guilds (id) ON DELETE CASCADE
);

CREATE OR REPLACE TABLE guild_config_auto_roles (
    guild_id VARCHAR(32) NOT NULL,
    enabled  BOOLEAN     NOT NULL DEFAULT FALSE,

    PRIMARY KEY (guild_id),
    FOREIGN KEY (guild_id) REFERENCES guilds (id) ON DELETE CASCADE
);

CREATE OR REPLACE TABLE guild_auto_roles (
    guild_id VARCHAR(32) NOT NULL,
    role_id VARCHAR(32) NOT NULL,

    PRIMARY KEY (guild_id, role_id),
    FOREIGN KEY (guild_id) REFERENCES guilds (id) ON DELETE CASCADE
);

CREATE OR REPLACE TABLE guild_config_citation (
    guild_id VARCHAR(32) NOT NULL,
    enabled  BOOLEAN     NOT NULL DEFAULT FALSE,
    channel  VARCHAR(32) DEFAULT NULL,

    PRIMARY KEY (guild_id),
    FOREIGN KEY (guild_id) REFERENCES guilds (id) ON DELETE CASCADE
);