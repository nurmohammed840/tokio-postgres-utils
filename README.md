It provide utilities to work with the [tokio-postgres](https://github.com/sfackler/rust-postgres) crate, specifically through the use of `FromRow` and `TryFromRow` derive macros.
These macros simplify the process of converting database rows into Rust structs.

# Installation

Add `tokio-postgres-utils` to your `Cargo.toml`:

```toml
[dependencies]
tokio-postgres = "0.7"
tokio-postgres-utils = "0.2"
```

## Example

```rust
use tokio_postgres_utils::FromRow;

#[derive(FromRow)]
struct User {
    id: i32,
    name: String,
}
```

Expand into something like:

```rust, ignore
use tokio_postgres::Row;

impl From<&Row> for User {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
        }
    }
}
```

## Field Attributes `#[column(..)]`

Several attributes can be specified to customize how each column in a row is read:

### `rename`

When the name of a field in Rust does not match the name of its corresponding column, you can use the rename attribute to specify the name that the field has in the row. For example:

```rust
use tokio_postgres_utils::FromRow;

#[derive(FromRow)]
struct User {
    id: i32,
    name: String,
    #[column(rename = "description")]
    about_me: String
}
```

Given a query such as:

```sql
SELECT id, name, description FROM users;
```

will read the content of the column `description` into the field `about_me`.


### `flatten`

If you want to handle a field that implements FromRow, you can use the flatten attribute to specify that you want it to use FromRow for parsing rather than the usual method. For example:

```rust
use tokio_postgres_utils::{TryFromRow, FromRow};

#[derive(FromRow)]
struct Address {
    country: String,
    city: String,
    road: String,
}

#[derive(TryFromRow)]
struct User {
    id: i32,
    name: String,
    #[column(flatten)]
    address: Address,
}
```

Given a query such as:

```sql
SELECT id, name, country, city, road FROM users;
```

### `skip`

The corresponding field should be ignored when mapping database query results and use default value.

This is particularly useful when you have fields in your struct that are not present in the query results or when you want to exclude certain fields from being populated by the query.


```rust
use tokio_postgres_utils::FromRow;

#[derive(Default)]
struct Address {
    user_name: String,
    street: String,
    city: String,
}

#[derive(FromRow)]
struct User {
    name: String,
    #[column(skip)]
    addresses: Vec<Address>,
}
```

Given a query such as:

```sql
SELECT name FROM users;
```
