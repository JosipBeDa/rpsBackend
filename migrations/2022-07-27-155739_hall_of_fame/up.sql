CREATE TABLE hall_of_fame (
    id SERIAL,
    user_id VARCHAR(36) UNIQUE NOT NULL,
    score INT NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);