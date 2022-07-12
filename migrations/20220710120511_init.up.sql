CREATE TABLE room (
    id serial PRIMARY KEY,
    name VARCHAR(50) NOT NULL
);

CREATE TABLE guest (
    id serial PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    multiaddr TEXT NOT NULL,
    room_id serial NOT NULL
);
