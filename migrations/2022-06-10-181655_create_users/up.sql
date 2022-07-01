CREATE TABLE users (
    id VARCHAR(36) UNIQUE,
    username VARCHAR (20) UNIQUE NOT NULL,
    "password" VARCHAR (255) NOT NULL,
    PRIMARY KEY (id)
);