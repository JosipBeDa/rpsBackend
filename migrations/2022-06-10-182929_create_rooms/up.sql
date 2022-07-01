CREATE TABLE rooms (
    id VARCHAR(36) UNIQUE,
    "name" VARCHAR (30) NOT NULL,
    "password" VARCHAR (255),
    "admin" VARCHAR(36),
    PRIMARY KEY (id),
    FOREIGN KEY ("admin") REFERENCES users(id)
);