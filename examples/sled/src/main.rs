use entity::{simple_ent, Ent, Id, SledDatabase};

#[simple_ent(serde, debug)]
struct User {
    name: String,
    age: u8,

    #[ent(edge(type = "Address"))]
    address: Id,
}

#[simple_ent(serde, debug)]
struct Address {
    street: String,
    city: String,
    zipcode: String,
    state: String,
}

fn main() {
    // Make our temporary sled::db
    let config = sled::Config::new().temporary(true);
    let db = config.open().expect("Failed to create database");

    // Define our wrapper (SledDatabase) around a tradition sled::db
    let db = SledDatabase::new(db);
    entity::global::set_db(db);

    let mut address = Address::build()
        .street("123 Some Street".to_string())
        .city("Some City".to_string())
        .zipcode("12345".to_string())
        .state("SW".to_string())
        .build()
        .unwrap();

    address.commit().unwrap();
    println!("{:?}", address);

    let mut user = User::build()
        .name("abc".to_string())
        .age(31)
        .address(address.id())
        .build()
        .unwrap();

    user.commit().unwrap();
    println!("{:?}", user);
}
