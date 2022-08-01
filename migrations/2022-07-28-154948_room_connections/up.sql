CREATE TABLE room_connections (
    room_id VARCHAR (36) NOT NULL,
    user_id VARCHAR (36) NOT NULL,
    FOREIGN KEY (room_id) REFERENCES rooms(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT UQ_room_user_pair PRIMARY KEY (room_id, user_id)
);