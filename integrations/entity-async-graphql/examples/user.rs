use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema};
use entity::*;
use entity_async_graphql::*;
use entity_inmemory::InmemoryDatabase;

#[simple_ent]
#[derive(EntObject, EntFilter)]
struct User {
    name: String,
    age: u8,

    #[ent(edge)]
    address: Address,
}

#[simple_ent]
#[derive(EntObject, EntFilter)]
struct Address {
    street: String,
    city: String,
}

/// Represents the root of our async-graphql query interface
struct RootQuery;

#[Object]
impl RootQuery {
    /// Supports retrieving an ent by its id or general filter
    async fn ent(
        &self,
        ctx: &Context<'_>,
        id: Option<Id>,
        filter: Option<GqlEntFilter>,
    ) -> async_graphql::Result<Vec<GqlDynEnt>> {
        let db = ctx.data::<DatabaseRc>()?;

        if let Some(id) = id {
            db.get_all(vec![id])
                .map(|ents| ents.into_iter().map(GqlDynEnt::from).collect())
                .map_err(|x| async_graphql::Error::new(x.to_string()))
        } else if let Some(filter) = filter {
            db.find_all(filter.into())
                .map(|ents| ents.into_iter().map(GqlDynEnt::from).collect())
                .map_err(|x| async_graphql::Error::new(x.to_string()))
        } else {
            Err(async_graphql::Error::new("Must provide one argument"))
        }
    }

    /// Supports retrieving an address by its id or specific filter
    async fn address(
        &self,
        ctx: &Context<'_>,
        id: Option<Id>,
        filter: Option<GqlAddressFilter>,
    ) -> async_graphql::Result<Vec<Address>> {
        let db = ctx.data::<DatabaseRc>()?;

        if let Some(id) = id {
            db.get_all_typed::<Address>(vec![id])
                .map_err(|x| async_graphql::Error::new(x.to_string()))
        } else if let Some(filter) = filter {
            db.find_all_typed::<Address>(filter.into())
                .map_err(|x| async_graphql::Error::new(x.to_string()))
        } else {
            Err(async_graphql::Error::new("Must provide one argument"))
        }
    }

    /// Supports retrieving a user by its id or specific filter
    async fn user(
        &self,
        ctx: &Context<'_>,
        id: Option<Id>,
        filter: Option<GqlUserFilter>,
    ) -> async_graphql::Result<Vec<User>> {
        let db = ctx.data::<DatabaseRc>()?;

        if let Some(id) = id {
            db.get_all_typed::<User>(vec![id])
                .map_err(|x| async_graphql::Error::new(x.to_string()))
        } else if let Some(filter) = filter {
            db.find_all_typed::<User>(filter.into())
                .map_err(|x| async_graphql::Error::new(x.to_string()))
        } else {
            Err(async_graphql::Error::new("Must provide one argument"))
        }
    }
}

fn main() {
    let db = preload_db();
    let schema = Schema::build(RootQuery, EmptyMutation, EmptySubscription)
        .data(WeakDatabaseRc::upgrade(&db).unwrap())
        .finish();

    // Do a generic search across all ents for a field "name" that ends with
    // the text "Flintstone" so we pull out the Flintstones
    run_query_and_print(
        &schema,
        r#"
        {
            ent(filter: { 
                fields: [
                    {
                        name: "name"
                        predicate: {
                            text_ends_with: "Flintstone"
                        }
                    }
                ]
            }) {
                id
                name: field(name: "name")
            }
        }
        "#,
    );

    // Now, do the same query using our user-specific filter
    run_query_and_print(
        &schema,
        r#"
        {
            user(filter: { name: { text_ends_with: "Flintstone" } }) {
                id
                name
            }
        }
        "#,
    );

    // Demonstrate finding users by their address
    run_query_and_print(
        &schema,
        r#"
        {
            user(filter: { 
                address: {
                    city: {
                        equals: "Bedrock"
                    }
                } 
            }) {
                id
                name
            }
        }
        "#,
    );
}

#[inline]
fn run_query_and_print(schema: &Schema<RootQuery, EmptyMutation, EmptySubscription>, input: &str) {
    let input = input.trim();
    println!("Query: {}", input);

    let res = futures::executor::block_on(schema.execute(input));
    match serde_json::to_string(&res) {
        Ok(x) => println!("Result: {}", x),
        Err(_) => println!("Result: {:?}", res),
    }
}

fn preload_db() -> WeakDatabaseRc {
    let db = InmemoryDatabase::default();
    entity::global::set_db(db);

    let flintstone_address = Address::build()
        .street("345 Cave Stone Road".to_string())
        .city("Bedrock".to_string())
        .finish_and_commit()
        .unwrap()
        .unwrap();

    let _ = User::build()
        .name("Fred Flintstone".to_string())
        .age(58)
        .address(flintstone_address.id())
        .finish_and_commit()
        .unwrap()
        .unwrap();

    let _ = User::build()
        .name("Wilma Flintstone".to_string())
        .age(57)
        .address(flintstone_address.id())
        .finish_and_commit()
        .unwrap()
        .unwrap();

    let rubble_address = Address::build()
        .street("123 Cave Stone Road".to_string())
        .city("Bedrock".to_string())
        .finish_and_commit()
        .unwrap()
        .unwrap();

    let _ = User::build()
        .name("Barney Rubble".to_string())
        .age(55)
        .address(rubble_address.id())
        .finish_and_commit()
        .unwrap()
        .unwrap();

    let _ = User::build()
        .name("Betty Rubble".to_string())
        .age(51)
        .address(rubble_address.id())
        .finish_and_commit()
        .unwrap()
        .unwrap();

    entity::global::db()
}
