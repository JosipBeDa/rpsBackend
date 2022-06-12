CREATE TABLE rooms (
    id VARCHAR(36) UNIQUE,
    "name" VARCHAR (30) NOT NULL,
    "password" VARCHAR (255),
    PRIMARY KEY (id)
);