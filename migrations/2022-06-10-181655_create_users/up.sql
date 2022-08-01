CREATE TABLE users (
    id VARCHAR(36) DEFAULT uuid_generate_v4() UNIQUE NOT NULL,
    username VARCHAR (20) UNIQUE NOT NULL,
    "password" VARCHAR (255) NOT NULL,
    PRIMARY KEY (id)
);