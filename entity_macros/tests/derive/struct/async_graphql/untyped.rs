use entity::*;

// NOTE: We need EntTypedEdges for now, but if the macro is updated to not
//       require it then we can remove that constraint
#[derive(Clone, Ent, EntTypedEdges, AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
struct TestEnt {
    #[ent(id)]
    id: Id,

    #[ent(database)]
    database: WeakDatabaseRc,

    #[ent(created)]
    created: u64,

    #[ent(last_updated)]
    last_updated: u64,

    #[ent(field, ext(async_graphql(filter_untyped)))]
    my_field: String,

    #[ent(edge(type = "TestEnt"), ext(async_graphql(filter_untyped)))]
    my_maybe_edge: Option<Id>,

    #[ent(edge(type = "TestEnt"), ext(async_graphql(filter_untyped)))]
    my_edge: Id,

    #[ent(edge(type = "TestEnt"), ext(async_graphql(filter_untyped)))]
    my_many_edges: Vec<Id>,
}

struct RootQuery;

#[async_graphql::Object]
impl RootQuery {
    async fn find(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: Option<Id>,
        filter: Option<GqlTestEntFilter>,
    ) -> async_graphql::Result<Vec<TestEnt>> {
        let db = ctx.data::<DatabaseRc>()?;

        if let Some(id) = id {
            match db.get_all_typed::<TestEnt>(vec![id]) {
                Ok(mut ents) => {
                    for ent in ents.iter_mut() {
                        ent.connect(DatabaseRc::downgrade(&db));
                    }
                    Ok(ents)
                }
                Err(x) => Err(async_graphql::Error::new(x.to_string())),
            }
        } else if let Some(filter) = filter {
            match db.find_all_typed::<TestEnt>(filter.into()) {
                Ok(mut ents) => {
                    for ent in ents.iter_mut() {
                        ent.connect(DatabaseRc::downgrade(&db));
                    }
                    Ok(ents)
                }
                Err(x) => Err(async_graphql::Error::new(x.to_string())),
            }
        } else {
            Err(async_graphql::Error::new("Must provide one argument"))
        }
    }
}

#[inline]
fn make_db() -> DatabaseRc {
    let db = InmemoryDatabase::default();
    let _ = db
        .insert(Box::new(TestEnt {
            id: 1,
            database: WeakDatabaseRc::new(),
            created: 111,
            last_updated: 0,
            my_field: "one".to_string(),
            my_maybe_edge: Some(2),
            my_edge: 2,
            my_many_edges: vec![2],
        }))
        .unwrap();

    // Delay next ent creation so we can ensure a different last_updated time
    std::thread::sleep(::std::time::Duration::from_millis(10));

    let _ = db
        .insert(Box::new(TestEnt {
            id: 2,
            database: WeakDatabaseRc::new(),
            created: 222,
            last_updated: 0,
            my_field: "two".to_string(),
            my_maybe_edge: Some(1),
            my_edge: 1,
            my_many_edges: vec![1],
        }))
        .unwrap();

    DatabaseRc::new(Box::new(db))
}

#[inline]
fn make_schema_and_db() -> (
    async_graphql::Schema<
        RootQuery,
        async_graphql::EmptyMutation,
        async_graphql::EmptySubscription,
    >,
    DatabaseRc,
) {
    let db = make_db();
    let schema = async_graphql::Schema::build(
        RootQuery,
        async_graphql::EmptyMutation,
        async_graphql::EmptySubscription,
    )
    .data(DatabaseRc::clone(&db))
    .finish();
    (schema, db)
}

#[inline]
fn make_schema(
) -> async_graphql::Schema<RootQuery, async_graphql::EmptyMutation, async_graphql::EmptySubscription>
{
    let (schema, _) = make_schema_and_db();
    schema
}

#[inline]
fn execute(
    schema: &async_graphql::Schema<
        RootQuery,
        async_graphql::EmptyMutation,
        async_graphql::EmptySubscription,
    >,
    input: &str,
) -> serde_json::Value {
    let input = input.trim();
    let res = futures::executor::block_on(schema.execute(input));
    let data = res.data.into_json().unwrap();
    assert_ne!(
        data,
        serde_json::Value::Null,
        "{}",
        serde_json::to_string_pretty(&res.errors).unwrap()
    );
    data
}

#[test]
fn produces_expected_filter_input_object() {
    let schema = make_schema();
    let result = execute(
        &schema,
        r#"{ 
            __type(name: "GqlTestEntFilter") { 
                inputFields { 
                    name
                    type { 
                        name
                        kind
                    }
                }
            } 
        }"#,
    );

    // NOTE: We are assuming that the introspection of fields yields them in
    //       the order they were defined!
    assert_eq!(
        result,
        serde_json::json!({
            "__type": {
                "inputFields": [
                    {
                        "name": "id",
                        "type": {
                            "kind": "INPUT_OBJECT",
                            "name": "GqlPredicateId",
                        }
                    },
                    {
                        "name": "created",
                        "type": {
                            "kind": "INPUT_OBJECT",
                            "name": "GqlPredicateU64",
                        }
                    },
                    {
                        "name": "last_updated",
                        "type": {
                            "kind": "INPUT_OBJECT",
                            "name": "GqlPredicateU64",
                        }
                    },
                    {
                        "name": "my_field",
                        "type": {
                            "kind": "INPUT_OBJECT",
                            "name": "GqlPredicateValue",
                        }
                    },
                    {
                        "name": "my_maybe_edge",
                        "type": {
                            "kind": "INPUT_OBJECT",
                            "name": "GqlEntFilter",
                        }
                    },
                    {
                        "name": "my_edge",
                        "type": {
                            "kind": "INPUT_OBJECT",
                            "name": "GqlEntFilter",
                        }
                    },
                    {
                        "name": "my_many_edges",
                        "type": {
                            "kind": "INPUT_OBJECT",
                            "name": "GqlEntFilter",
                        }
                    }
                ]
            }
        })
    );
}

#[test]
fn produces_expected_ent_output_object() {
    let schema = make_schema();
    let result = execute(
        &schema,
        r#"{ 
            __type(name: "TestEnt") { 
                fields { 
                    name
                    type { 
                        name
                        kind
                        ofType { 
                            name
                            kind 
                            ofType {
                                name
                                kind
                                ofType {
                                    name
                                    kind
                                }
                            }
                        } 
                    }
                }
            } 
        }"#,
    );

    // NOTE: We are assuming that the introspection of fields yields them in
    //       the order they were defined!
    assert_eq!(
        result,
        serde_json::json!({
            "__type": {
                "fields": [
                    {
                        "name": "id",
                        "type": {
                            "name": null,
                            "kind": "NON_NULL",
                            "ofType": {
                                "name": "Int",
                                "kind": "SCALAR",
                                "ofType": null,
                            }
                        }
                    },
                    {
                        "name": "type",
                        "type": {
                            "name": null,
                            "kind": "NON_NULL",
                            "ofType": {
                                "name": "String",
                                "kind": "SCALAR",
                                "ofType": null,
                            }
                        }
                    },
                    {
                        "name": "created",
                        "type": {
                            "name": null,
                            "kind": "NON_NULL",
                            "ofType": {
                                "name": "Int",
                                "kind": "SCALAR",
                                "ofType": null,
                            }
                        }
                    },
                    {
                        "name": "last_updated",
                        "type": {
                            "name": null,
                            "kind": "NON_NULL",
                            "ofType": {
                                "name": "Int",
                                "kind": "SCALAR",
                                "ofType": null,
                            }
                        }
                    },
                    {
                        "name": "my_field",
                        "type": {
                            "name": null,
                            "kind": "NON_NULL",
                            "ofType": {
                                "name": "String",
                                "kind": "SCALAR",
                                "ofType": null,
                            }
                        }
                    },
                    {
                        "name": "id_for_my_maybe_edge",
                        "type": {
                            "name": "Int",
                            "kind": "SCALAR",
                            "ofType": null,
                        }
                    },
                    {
                        "name": "id_for_my_edge",
                        "type": {
                            "name": null,
                            "kind": "NON_NULL",
                            "ofType": {
                                "name": "Int",
                                "kind": "SCALAR",
                                "ofType": null,
                            }
                        }
                    },
                    {
                        "name": "ids_for_my_many_edges",
                        "type": {
                            "name": null,
                            "kind": "NON_NULL",
                            "ofType": {
                                "name": null,
                                "kind": "LIST",
                                "ofType": {
                                    "name": null,
                                    "kind": "NON_NULL",
                                    "ofType": {
                                        "name": "Int",
                                        "kind": "SCALAR",
                                    },
                                },
                            }
                        }
                    },
                    {
                        "name": "my_maybe_edge",
                        "type": {
                            "name": "TestEnt",
                            "kind": "OBJECT",
                            "ofType": null,
                        }
                    },
                    {
                        "name": "my_edge",
                        "type": {
                            "name": null,
                            "kind": "NON_NULL",
                            "ofType": {
                                "name": "TestEnt",
                                "kind": "OBJECT",
                                "ofType": null,
                            },
                        }
                    },
                    {
                        "name": "my_many_edges",
                        "type": {
                            "name": null,
                            "kind": "NON_NULL",
                            "ofType": {
                                "name": null,
                                "kind": "LIST",
                                "ofType": {
                                    "name": null,
                                    "kind": "NON_NULL",
                                    "ofType": {
                                        "name": "TestEnt",
                                        "kind": "OBJECT",
                                    },
                                },
                            },
                        }
                    },
                ]
            }
        })
    );
}
