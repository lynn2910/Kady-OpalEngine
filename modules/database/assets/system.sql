DROP TABLE IF EXISTS cookies_user_quiz;
DROP TABLE IF EXISTS cookies_quiz_answers;
DROP TABLE IF EXISTS cookies_quiz_questions;

CREATE OR REPLACE TABLE cookies_quiz_questions (
    id CHAR(36) PRIMARY KEY NOT NULL,
    category VARCHAR(24) NOT NULL
);

CREATE OR REPLACE TABLE cookies_quiz_answers (
    id CHAR(36) NOT NULL,
    answer VARCHAR(254) NOT NULL,
    
    FOREIGN KEY (id) REFERENCES cookies_quiz_questions (id)
);


# Used to store what question the user was given
CREATE OR REPLACE TABLE cookies_user_quiz (
    id CHAR(36) NOT NULL,
    user VARCHAR(32) NOT NULL,
    date DATE NOT NULL DEFAULT CURDATE(),
    completed BOOLEAN NOT NULL DEFAULT false,

    PRIMARY KEY (id, user),
    FOREIGN KEY (id) REFERENCES cookies_quiz_questions (id),
    FOREIGN KEY (user) REFERENCES users(id)
);