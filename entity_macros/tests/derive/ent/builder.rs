use derivative::Derivative;
use entity::{Database, Ent, Id, Value};
use std::convert::TryFrom;

#[test]
fn produces_an_error_enum_for_each_struct_field() {
    #[derive(Clone, Ent)]
    #[ent(builder)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(field)]
        field1: u32,

        #[ent(field)]
        field2: String,

        #[ent(edge(type = "TestEnt"))]
        edge1: Option<Id>,

        #[ent(edge(type = "TestEnt"))]
        edge2: Id,

        #[ent(edge(type = "TestEnt"))]
        edge3: Vec<Id>,
    }

    assert_eq!(TestEntBuilderError::MissingId.to_string(), "Missing id");
    assert_eq!(
        TestEntBuilderError::MissingDatabase.to_string(),
        "Missing database"
    );
    assert_eq!(
        TestEntBuilderError::MissingCreated.to_string(),
        "Missing created"
    );
    assert_eq!(
        TestEntBuilderError::MissingLastUpdated.to_string(),
        "Missing last_updated"
    );
    assert_eq!(
        TestEntBuilderError::MissingField1.to_string(),
        "Missing field1"
    );
    assert_eq!(
        TestEntBuilderError::MissingField2.to_string(),
        "Missing field2"
    );
    assert_eq!(
        TestEntBuilderError::MissingEdge1.to_string(),
        "Missing edge1"
    );
    assert_eq!(
        TestEntBuilderError::MissingEdge2.to_string(),
        "Missing edge2"
    );
    assert_eq!(
        TestEntBuilderError::MissingEdge3.to_string(),
        "Missing edge3"
    );
}

#[test]
fn default_returns_a_builder_with_all_fields_set_to_none() {
    #[derive(Clone, Ent)]
    #[ent(builder)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(field)]
        field1: u32,

        #[ent(field)]
        field2: String,

        #[ent(edge(type = "TestEnt"))]
        edge1: Option<Id>,

        #[ent(edge(type = "TestEnt"))]
        edge2: Id,

        #[ent(edge(type = "TestEnt"))]
        edge3: Vec<Id>,
    }

    let builder = TestEntBuilder::default();
    assert_eq!(builder.id, None);
    assert_eq!(builder.database.is_none(), true);
    assert_eq!(builder.created, None);
    assert_eq!(builder.last_updated, None);
    assert_eq!(builder.field1, None);
    assert_eq!(builder.field2, None);
    assert_eq!(builder.edge1, None);
    assert_eq!(builder.edge2, None);
    assert_eq!(builder.edge3, None);
}

#[test]
fn build_fails_when_struct_field_is_not_set() {
    #[derive(Clone, Derivative, Ent)]
    #[derivative(Debug)]
    #[ent(builder)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[derivative(Debug = "ignore")]
        #[ent(database)]
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(field)]
        field1: u32,

        #[ent(field)]
        field2: String,

        #[ent(edge(type = "TestEnt"))]
        edge1: Option<Id>,

        #[ent(edge(type = "TestEnt"))]
        edge2: Id,

        #[ent(edge(type = "TestEnt"))]
        edge3: Vec<Id>,
    }

    assert_eq!(
        TestEntBuilder::default()
            .database(None)
            .created(0)
            .last_updated(0)
            .field1(0)
            .field2(String::from("test"))
            .edge1(None)
            .edge2(0)
            .edge3(vec![])
            .build()
            .unwrap_err(),
        TestEntBuilderError::MissingId,
    );

    assert_eq!(
        TestEntBuilder::default()
            .id(0)
            .created(0)
            .last_updated(0)
            .field1(0)
            .field2(String::from("test"))
            .edge1(None)
            .edge2(0)
            .edge3(vec![])
            .build()
            .unwrap_err(),
        TestEntBuilderError::MissingDatabase,
    );

    assert_eq!(
        TestEntBuilder::default()
            .id(0)
            .database(None)
            .last_updated(0)
            .field1(0)
            .field2(String::from("test"))
            .edge1(None)
            .edge2(0)
            .edge3(vec![])
            .build()
            .unwrap_err(),
        TestEntBuilderError::MissingCreated,
    );

    assert_eq!(
        TestEntBuilder::default()
            .id(0)
            .database(None)
            .created(0)
            .field1(0)
            .field2(String::from("test"))
            .edge1(None)
            .edge2(0)
            .edge3(vec![])
            .build()
            .unwrap_err(),
        TestEntBuilderError::MissingLastUpdated,
    );

    assert_eq!(
        TestEntBuilder::default()
            .id(0)
            .database(None)
            .created(0)
            .last_updated(0)
            .field2(String::from("test"))
            .edge1(None)
            .edge2(0)
            .edge3(vec![])
            .build()
            .unwrap_err(),
        TestEntBuilderError::MissingField1,
    );

    assert_eq!(
        TestEntBuilder::default()
            .id(0)
            .database(None)
            .created(0)
            .last_updated(0)
            .field1(0)
            .edge1(None)
            .edge2(0)
            .edge3(vec![])
            .build()
            .unwrap_err(),
        TestEntBuilderError::MissingField2,
    );

    assert_eq!(
        TestEntBuilder::default()
            .id(0)
            .database(None)
            .created(0)
            .last_updated(0)
            .field1(0)
            .field2(String::from("test"))
            .edge2(0)
            .edge3(vec![])
            .build()
            .unwrap_err(),
        TestEntBuilderError::MissingEdge1,
    );

    assert_eq!(
        TestEntBuilder::default()
            .id(0)
            .database(None)
            .created(0)
            .last_updated(0)
            .field1(0)
            .field2(String::from("test"))
            .edge1(None)
            .edge3(vec![])
            .build()
            .unwrap_err(),
        TestEntBuilderError::MissingEdge2,
    );

    assert_eq!(
        TestEntBuilder::default()
            .id(0)
            .database(None)
            .created(0)
            .last_updated(0)
            .field1(0)
            .field2(String::from("test"))
            .edge1(None)
            .edge2(0)
            .build()
            .unwrap_err(),
        TestEntBuilderError::MissingEdge3,
    );
}

#[test]
fn build_succeeds_when_all_struct_fields_are_set() {
    #[derive(Clone, Derivative, Ent)]
    #[derivative(Debug)]
    #[ent(builder)]
    struct TestEnt {
        #[ent(id)]
        id: Id,

        #[derivative(Debug = "ignore")]
        #[ent(database)]
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(field)]
        field1: u32,

        #[ent(field)]
        field2: String,

        #[ent(edge(type = "TestEnt"))]
        edge1: Option<Id>,

        #[ent(edge(type = "TestEnt"))]
        edge2: Id,

        #[ent(edge(type = "TestEnt"))]
        edge3: Vec<Id>,
    }

    let ent = TestEntBuilder::default()
        .id(1)
        .database(None)
        .created(2)
        .last_updated(3)
        .field1(4)
        .field2(String::from("test"))
        .edge1(Some(5))
        .edge2(6)
        .edge3(vec![7, 8])
        .build()
        .expect("Failed to build ent!");
    assert_eq!(ent.id, 1);
    assert_eq!(ent.created, 2);
    assert_eq!(ent.last_updated, 3);
    assert_eq!(ent.field1, 4);
    assert_eq!(ent.field2, String::from("test"));
    assert_eq!(ent.edge1, Some(5));
    assert_eq!(ent.edge2, 6);
    assert_eq!(ent.edge3, vec![7, 8]);
}

#[test]
fn supports_generic_fields() {
    #[derive(Clone, Ent)]
    #[ent(builder)]
    struct TestEnt<T>
    where
        T: TryFrom<Value, Error = &'static str> + Into<Value> + Clone + 'static,
    {
        #[ent(id)]
        id: Id,

        #[ent(database)]
        database: Option<Box<dyn Database>>,

        #[ent(created)]
        created: u64,

        #[ent(last_updated)]
        last_updated: u64,

        #[ent(field)]
        generic_field: T,
    }

    let builder = TestEntBuilder::default();
    assert_eq!(builder.id, None);
    assert_eq!(builder.database.is_none(), true);
    assert_eq!(builder.created, None);
    assert_eq!(builder.last_updated, None);
    assert_eq!(builder.generic_field, None);

    let ent = builder
        .id(0)
        .database(None)
        .created(0)
        .last_updated(0)
        .generic_field(3)
        .build()
        .expect("Failed to create with generic field");
    assert_eq!(ent.generic_field, 3);
}
