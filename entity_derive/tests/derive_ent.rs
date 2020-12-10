use entity::{Database, Ent, Id};
use serde::{Deserialize, Serialize};

#[derive(Clone, Ent, Serialize, Deserialize)]
#[ent(typetag)]
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

    #[ent(field(indexed))]
    title: String,
    // #[ent(field)]
    // url: String,

    // #[ent(edge(shallow, type = "ContentEnt"))]
    // header: Id,

    // #[ent(edge(deep, type = "ContentEnt"))]
    // subheader: Option<Id>,

    // #[ent(edge(type = "ContentEnt"))]
    // paragraphs: Vec<Id>,
}

#[derive(Clone, Ent, Serialize, Deserialize)]
#[ent(typetag)]
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
}
