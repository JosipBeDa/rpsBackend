CREATE TABLE user_room_junction (
    id SERIAL,
    user_id VARCHAR(36)NOT NULL,
    room_id VARCHAR(36),
    user_is_admin BOOLEAN NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (room_id) REFERENCES rooms(id)
);