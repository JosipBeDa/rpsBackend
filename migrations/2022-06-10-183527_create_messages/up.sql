CREATE TABLE messages (
    id SERIAL,
    sender_id VARCHAR (36) NOT NULL,
    rec_user VARCHAR (36),
    rec_room VARCHAR(36),
    body VARCHAR (500),
    time_sent TIMESTAMPTZ,
    PRIMARY KEY (id),
    FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (rec_user) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (rec_room) REFERENCES rooms(id) ON DELETE CASCADE
);