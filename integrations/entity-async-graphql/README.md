# entity-async-graphql

Provides data structures to expose `entity` objects via `async-graphql`.

## Example

Leveraging `macros` feature alongside the `entity-inmemory` and `futures`
crates:

```rust
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
  friends: Vec<User>,
}

struct RootQuery;

#[Object]
impl RootQuery {
  async fn users(
    &self,
    ctx: &Context<'_>,
    filter: GqlUserFilter,
  ) -> async_graphql::Result<Vec<User>> {
    let db = ctx.data::<DatabaseRc>()?;

    db.find_all_typed::<User>(filter.into())
        .map_err(|x| async_graphql::Error::new(x.to_string()))
  }
}

fn main() {
  // Create an empty database and convert from InmemoryDatabase -> DatabaseRc
  let db = db_to_rc(InmemoryDatabase::default());

  // Make a user and write it into the database
  let user = User::build()
    .name(String::from("Fred Flintstone"))
    .age(38)
    .friends(Vec::new())
    .finish()
    .expect("Built user");
  let _ = db.insert_typed(user).expect("Wrote to database");

  // Define our GraphQL schema and add our database as context data
  let schema = Schema::build(RootQuery, EmptyMutation, EmptySubscription)
      .data(db)
      .finish();

  // Execute a GraphQL query and print the results
  let res = futures::executor::block_on(schema.execute(r#"
    {
      user(filter: { name: { text_ends_with: "Flintstone" } }) {
        id
        name
      }
    }
  "#));

  println!("{:#?}", res);
}
```

## Feature Flags

* **`macros`** - provides macro support for generating needed
  `async-graphql` code for new entities
