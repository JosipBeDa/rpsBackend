CREATE TABLE messages (
    id SERIAL,
    sender_id VARCHAR (36) NOT NULL,
    room_id VARCHAR (36),
    body VARCHAR (500),
    time_sent TIMESTAMPTZ,
    PRIMARY KEY (id),
    FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (room_id) REFERENCES rooms(id) ON DELETE CASCADE
);