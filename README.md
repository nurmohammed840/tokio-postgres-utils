It provide utilities to work with the [tokio-postgres](https://github.com/sfackler/rust-postgres) crate, specifically through the use of `FromRow` and `TryFromRow` derive macros.
These macros simplify the process of converting database rows into Rust structs.

# Installation

Add `tokio-postgres-utils` to your `Cargo.toml`:

```toml
[dependencies]
tokio-postgres = "0.7"
tokio-postgres-utils = "0.1"
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

```rust
use tokio_postgres::Row;

impl From<&Row> for User {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
        }
    }
}