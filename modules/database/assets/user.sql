DROP TABLE IF EXISTS `user_badges`;
DROP TABLE IF EXISTS `user_xp`;
DROP TABLE IF EXISTS `users_marriage`;
DROP TABLE IF EXISTS `user_biography`;
DROP TABLE IF EXISTS user_cookies;
DROP TABLE IF EXISTS `users`;

# Table that contains the user's settings.
CREATE OR REPLACE TABLE `users`
(
    id                    VARCHAR(32) NOT NULL,
    # Used to auto-delete the data (RGPD friendly)
    last_seen             DATETIME    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_edited_timestamp DATETIME    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    # Used to know if the user want to receive private messages
    send_private_messages BOOLEAN     NOT NULL DEFAULT TRUE,
    PRIMARY KEY (id)
);

# Table that contain the user's XP and level.
# XP is the amount of XP the user has.
CREATE OR REPLACE TABLE `user_xp`
(
    user VARCHAR(32),
    xp   INT NOT NULL DEFAULT 0,
    lvl  INT NOT NULL DEFAULT 0,
    FOREIGN KEY (user) REFERENCES users (id) ON DELETE CASCADE,
    PRIMARY KEY (user)
);

# Contain each user with their badges.
CREATE OR REPLACE TABLE `user_badges`
(
    user  VARCHAR(32),
    # The badges are stored as an integer, with each bit representing a badge.
    badge BIGINT UNSIGNED NOT NULL,
    FOREIGN KEY (user) REFERENCES users (id) ON DELETE CASCADE,
    PRIMARY KEY (user)
);

# Contain each marriage between users.
CREATE OR REPLACE TABLE `users_marriage`
(
    user1      VARCHAR(32),
    user2      VARCHAR(32),
    timestamp  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (user1) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (user2) REFERENCES users (id) ON DELETE CASCADE,
    PRIMARY KEY (user1, user2),
    # The user1 and user2 must be different.
    CHECK (user1 != user2)
);

# Contain each user's biography.
# Must be less than 255 characters and unique for each user.
CREATE OR REPLACE TABLE `user_biography`
(
    user      VARCHAR(32),
    biography VARCHAR(255),
    FOREIGN KEY (user) REFERENCES users (id) ON DELETE CASCADE,
    PRIMARY KEY (user)
);



# Contain each cookie given by a user to another.
CREATE OR REPLACE TABLE user_cookies
(
    # The user_from is the user that give the reputation.
    user_from VARCHAR(32) NOT NULL,
    # The user_to is the user that receive the reputation.
    user_to   VARCHAR(32) NOT NULL,
    guild     VARCHAR(32),
    # The timestamp is used primarily for the reputation cooldown.
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (user_from) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (user_to) REFERENCES users (id) ON DELETE CASCADE,
    # The user_from and user_to must be different.
    CHECK (user_from != user_to)
);


CREATE OR REPLACE TABLE user_cookie_nuggets
(
    user VARCHAR(32) NOT NULL,
    nuggets INT UNSIGNED NOT NULL DEFAULT 0,
    PRIMARY KEY (user),

    FOREIGN KEY (user) REFERENCES users (id) ON DELETE CASCADE
);




# INSERT INTO user_cookies (user_from, user_to, guild)
# VALUES
#     ('816708235440029706', '782164174821523467', '672873780481359892'),
#     ('782164174821523467', '816708235440029706', '672873780481359892'),
#     ('816708235440029706', '782164174821523467', '672873780481359892'),
#     ('782164174821523467', '816708235440029706', '672873780481359892');

# /* GET ALL */
# SELECT * FROM `user_cookies` WHERE `user_to` = '782164174821523467';

# /* GET LAST */
# SELECT * FROM `user_cookies` WHERE `user_to` = '782164174821523467' ORDER BY `timestamp` DESC LIMIT 1;

# /* GET TOP 10 GLOBAL */
# SELECT user_to, COUNT(*) as cookies FROM user_cookies
# GROUP BY user_to
# ORDER BY cookies DESC
# LIMIT 10;

# /* GET TOP 10 guild */
# SELECT user_to, COUNT(*) as cookies FROM user_cookies
# WHERE guild = '672873780481359892'
# GROUP BY user_to
# ORDER BY cookies DESC
# LIMIT 10;

# /* GET USER RANKING GLOBAL */
# SELECT user_to, cookies, user_rank
# FROM (
#          SELECT user_to, cookies,
#                 RANK() OVER (ORDER BY cookies DESC) as user_rank
#          FROM (
#                   SELECT user_to, COUNT(*) as cookies
#                   FROM user_cookies
#                   GROUP BY user_to
#               ) as r
#      ) as ranked_users
# WHERE user_to = '782164174821523467';

# SELECT user_to, reputation_count, user_rank FROM (SELECT user_to, reputation_count, RANK() OVER (ORDER BY reputation_count DESC) as user_rank FROM ( SELECT user_to, COUNT(*) as reputation_count FROM user_cookies GROUP BY user_to ) as r ) as ranked_users WHERE user_to = ?;

# /* GET USER RANK GUILD */
# SELECT user_to, reputation_count, user_rank
# FROM (
#          SELECT user_to, reputation_count,
#                 RANK() OVER (ORDER BY reputation_count DESC) as user_rank
#          FROM (
#                   SELECT user_to, COUNT(*) as reputation_count
#                   FROM user_cookies
#                   WHERE guild = 'a'
#                   GROUP BY user_to
#               ) as r
#      ) as ranked_users
# WHERE user_to = '782164174821523467';


# SELECT user_to, reputation_count, user_rank FROM (SELECT user_to, reputation_count, RANK() OVER (ORDER BY reputation_count DESC) as user_rank FROM (SELECT user_to, COUNT(*) as reputation_count FROM user_cookies WHERE guild = 'a' GROUP BY user_to) as r) as ranked_users WHERE user_to = '782164174821523467';
