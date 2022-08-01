CREATE TABLE messages (
    id VARCHAR(36) UNIQUE NOT NULL,
    sender_id VARCHAR (36) NOT NULL,
    receiver_user VARCHAR (36),
    receiver_room VARCHAR (36),
    content VARCHAR (2000),
    "timestamp" TIMESTAMPTZ,
    "read" BOOLEAN,
    PRIMARY KEY (id),
    FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (receiver_user) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (receiver_room) REFERENCES rooms(id) ON DELETE CASCADE
);