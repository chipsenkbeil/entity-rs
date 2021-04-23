mod r#enum;
mod r#struct;

pub use r#enum::{Ent as EnumEnt, EntVariant as EnumEntVariant};
pub use r#struct::{
    Ent as StructEnt, EntEdge as StructEntEdge,
    EntEdgeDeletionPolicy as StructEntEdgeDeletionPolicy, EntEdgeKind as StructEntEdgeKind,
    EntField as StructEntField,
};
