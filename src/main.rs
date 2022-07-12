#[macro_use] extern crate rocket;

use rocket::Build;
use std::fmt::Debug;
use tokio::runtime::Builder;
use futures::TryStreamExt;
use futures::executor::block_on;
use rocket::Rocket;
use rocket::serde::{json::Json, Serialize, Deserialize};
use rocket_db_pools::{Database, Connection};
use rocket_db_pools::sqlx::{self, Row};

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct Room {
    id: Option<i32>,
    name: String,
    guests: Vec<Guest>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct Guest {
    id: Option<i32>,
    name: String,
    multiaddr: String,
    room_id: i32,
}

#[derive(Database)]
#[database("postgres")]
pub struct Rooms(sqlx::PgPool);

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build().attach(Rooms::init()).mount("/", routes![get_rooms, add_room, join_room])
}

fn get_connection() -> sqlx::PgPool {
    let builder = Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let connect_future = sqlx::PgPool::connect("postgres://postgres:password@db/postgres");
    builder.block_on(connect_future).unwrap()
}

#[get("/rooms")]
async fn get_rooms(mut db: Connection<Rooms>) -> Json<Vec<Room>> {
    let mut query = sqlx::query("SELECT room.id, room.name FROM room").fetch(&mut *db);
    let mut rooms = vec![];
    while let Some(room) = query.try_next().await.unwrap() {
        let mut guests = vec![];
        let room_id: i32 = room.try_get(0).unwrap();
        let query_str = format!("SELECT id, name, multiaddr FROM guest WHERE room_id = {}", room_id);
        let query_str = query_str.as_str();
        let connect_future = sqlx::PgPool::connect("postgres://postgres:password@db/postgres");
        let pool = block_on(connect_future).unwrap();
        let mut guest_query = sqlx::query(query_str).fetch(&pool);
        while let Some(guest) = guest_query.try_next().await.unwrap() {
            let id = guest.try_get(0).unwrap();
            let name = guest.try_get(1).unwrap();
            let multiaddr = guest.try_get(2).unwrap();
            let guest = Guest { id, name, multiaddr, room_id };

            guests.push(guest);
        }
        let room = Room{id: Some(room_id), name: room.try_get(1).unwrap(), guests};
        rooms.push(room);
    }

    Json(rooms)
}

#[post("/rooms", data = "<room>")]
async fn add_room(mut db: Connection<Rooms>, room: Json<Room>) -> Json<Room> {
    sqlx::query(format!("INSERT INTO room (name) VALUES ('{}')", room.name).as_str()).execute(&mut *db).await.unwrap();
    let id: Option<i32> = sqlx::query(format!("SELECT id FROM room WHERE name = '{}'", room.name).as_str()).fetch_one(&mut *db).await.unwrap().try_get(0).unwrap();

    let db_guests = sqlx::query(format!("SELECT id, name, multiaddr FROM guest WHERE room_id = {}", id.unwrap()).as_str()).fetch_all(&mut *db).await.unwrap();

    let mut guests = vec![];

    for guest in db_guests {
        let id = guest.get(0);
        let name = guest.get(1);
        let multiaddr = guest.get(2);

        let guest = Guest { id, name, multiaddr, room_id: id.unwrap() };
        guests.push(guest);
    }

    Json(Room{id, name: room.name.clone(), guests})
}

#[post("/join", data = "<guest>")]
async fn join_room(mut db: Connection<Rooms>, guest: Json<Guest>) -> Json<Guest> {
    sqlx::query(format!("INSERT INTO guest (name, multiaddr, room_id) VALUES ('{}', '{}', {})", guest.name, guest.multiaddr, guest.room_id).as_str()).execute(&mut *db).await.unwrap();
    Json(guest.0)
}

fn test_teardown() {
    let builder = Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let connect_future = sqlx::PgPool::connect("postgres://postgres:password@db/postgres");
    let pool = builder.block_on(connect_future).unwrap();
    let mut queries = vec![];
    queries.push(sqlx::query("DELETE FROM guest").execute(&pool));
    queries.push(sqlx::query("DELETE FROM room").execute(&pool));
    for query in queries {
        builder.block_on(query).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::rocket;
    use super::test_teardown;
    use rocket::local::blocking::Client;
    use rocket::http::Status;

    #[test]
    fn test_get_rooms() {
        let client = Client::tracked(rocket()).unwrap();

        let room = super::Room {id: None, name: String::from("dupa"), guests: vec![]};
        let room2 = super::Room {id: None, name: String::from("pizda"), guests: vec![]};
        client.post(uri!(super::add_room)).json(&room).dispatch();
        client.post(uri!(super::add_room)).json(&room2).dispatch();

        let response = client.get(uri!(super::get_rooms)).dispatch();


        let json = response.into_json::<Vec<super::Room>>().unwrap();

        assert_eq!(json.len(), 2);
        assert_eq!(json.first().unwrap().name, room.name);

        let room_id = json.first().unwrap().id.unwrap();
        assert_eq!(json.into_iter().nth(1).unwrap().name, room2.name);

        let guest = super::Guest { id: None, name: String::from("dupeczka"), multiaddr: String::from("some/multiaddr"), room_id };

        client.post(uri!(super::join_room)).json(&guest).dispatch();

        let response = client.get(uri!(super::get_rooms)).dispatch();

        let json = response.into_json::<Vec<super::Room>>().unwrap();

        test_teardown();

        dbg!(json);


    }

    #[test]
    fn test_add_room() {
        let client = Client::tracked(rocket()).unwrap();
        let room = super::Room {id: None, name: String::from("cipa"), guests: vec![]};
        let response = client.post(uri!(super::add_room)).json(&room).dispatch();

        let result_room = response.into_json::<super::Room>().unwrap();

        test_teardown();

        assert_eq!(result_room.name, room.name);
    }

    #[test]
    fn test_join_room() {
        let client = Client::tracked(rocket()).unwrap();

        let room = super::Room {id: None, name: String::from("cipa"), guests: vec![]};
        let response = client.post(uri!(super::add_room)).json(&room).dispatch();

        let room_id = response.into_json::<super::Room>().unwrap().id.unwrap();

        let guest = super::Guest { id: None, name: String::from("dupeczka"), multiaddr: String::from("multi/addr"), room_id };

        let response = client.post(uri!(super::join_room)).json(&guest).dispatch();

        test_teardown();

        assert_eq!(response.status(), Status::Ok);
    }
}
