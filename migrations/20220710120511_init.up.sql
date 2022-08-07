CREATE TABLE room (
    id serial PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    CONSTRAINT name_unique UNIQUE (name)
);

CREATE TABLE guest (
    id serial PRIMARY KEY,
    name TEXT NOT NULL,
    multiaddr TEXT NOT NULL,
    room_id serial NOT NULL,
    CONSTRAINT fk_room
        FOREIGN KEY(room_id)
            REFERENCES room(id),
    CONSTRAINT unique_multiaddr UNIQUE multiaddr,
    CONSTRAINT uniq_name UNIQUE name
);

INSERT INTO room (name) VALUES ('main');
