CREATE TABLE guest (
    id serial PRIMARY KEY,
    name TEXT NOT NULL,
    multiaddr TEXT NOT NULL,
    room_id serial NOT NULL,
    CONSTRAINT fk_room
        FOREIGN KEY(room_id)
            REFERENCES room(id)
);
