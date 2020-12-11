use entity::{Database, Ent, Id, InmemoryDatabase};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct CustomInt(u32);

impl From<CustomInt> for entity::Value {
    fn from(x: CustomInt) -> Self {
        Self::from(x.0)
    }
}

impl std::convert::TryFrom<entity::Value> for CustomInt {
    type Error = &'static str;

    fn try_from(value: entity::Value) -> Result<Self, Self::Error> {
        Ok(Self(u32::try_from(value)?))
    }
}

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

    #[ent(field(indexed))]
    title: String,

    #[ent(field)]
    url: String,

    #[ent(field)]
    custom: CustomInt,

    #[ent(field)]
    list: Vec<String>,

    #[ent(field)]
    opt: Option<String>,

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
