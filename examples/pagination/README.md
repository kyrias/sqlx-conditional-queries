# Pagination Example

## Setup

1. Declare the database URL

    ```bash
    export DATABASE_URL="sqlite:/path/to/pagination.db"
    ```

2. Create the database and run migrations

    ```bash
    sqlx database create
    sqlx migrate run
    ```

## Usage Examples

List all items:
```bash
cargo run
```

Filter items by name (contains "10"):
```bash
cargo run -- --name-like 10
```

Get first 10 items:
```bash
cargo run -- --limit 10
```

Get 10 items starting from the 20th item:
```bash
cargo run -- --limit 10 --offset 20
```
