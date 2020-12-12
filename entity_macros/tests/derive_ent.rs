use entity::{Database, Ent, Id, InmemoryDatabase, Value};
use serde::{Deserialize, Serialize};

#[derive(Clone, Value, Serialize, Deserialize)]
pub struct CustomString(String);

#[derive(Clone, Value, Serialize, Deserialize)]
pub struct CustomType {
    x: u32,
    y: u64,
    z: Option<CustomString>,
    a: Vec<CustomType>,
}

#[derive(Clone, Value, Serialize, Deserialize)]
pub struct CustomUnit;

#[derive(Clone, Ent, Serialize, Deserialize)]
#[ent(typetag, typed_methods, builder, query)]
pub struct PageEnt {
    #[ent(id)]
    id: Id,

    #[ent(database)]
    #[serde(skip)]
    database: Option<Box<dyn Database>>,

    #[ent(created)]
    created: u64,

    #[ent(last_updated)]
    last_updated: u64,

    #[ent(field(indexed, mutable))]
    title: String,

    #[ent(field(mutable))]
    url: String,

    #[ent(field(mutable))]
    c1: CustomString,

    #[ent(field)]
    c2: CustomType,

    #[ent(field)]
    c3: CustomUnit,

    #[ent(field)]
    list: Vec<String>,

    #[ent(edge(shallow, type = "ContentEnt"))]
    header: Id,

    #[ent(edge(deep, type = "ContentEnt"))]
    subheader: Option<Id>,

    #[ent(edge(type = "ContentEnt"))]
    paragraphs: Vec<Id>,
}

#[derive(Clone, Ent, Serialize, Deserialize)]
#[ent(typetag, typed_methods, builder, query)]
pub struct ContentEnt {
    #[ent(id)]
    id: Id,

    #[ent(database)]
    #[serde(skip)]
    database: Option<Box<dyn Database>>,

    #[ent(created)]
    created: u64,

    #[ent(last_updated)]
    last_updated: u64,

    #[ent(edge(shallow, type = "PageEnt"))]
    page: Id,
}
