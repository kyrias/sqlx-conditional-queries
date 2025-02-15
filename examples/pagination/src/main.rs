use clap::Parser;
use sqlx::sqlite::SqlitePool;
use sqlx_conditional_queries::conditional_query_as;

#[derive(Debug)]
struct OutputType {
    id: i64,
    name: String,
    description: String,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    limit: Option<i32>,

    #[arg(long)]
    offset: Option<i32>,

    #[arg(long)]
    name_like: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let args = Args::parse();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable is not set");

    let pool = SqlitePool::connect(&database_url).await?;

    let (limit, offset, name_like) = (args.limit, args.offset, args.name_like);
    let results = conditional_query_as!(
        OutputType,
        r#"
            SELECT id, name, description
            FROM items
            {#where_clause}
            {#limit}
            {#offset}
        "#,
        #where_clause = match &name_like {
            Some(_) => "WHERE name LIKE '%' || {name_like} || '%'",
            None => "",
        },
        #(limit, offset) = match (limit, offset) {
            (Some(_), Some(_)) => ("LIMIT {limit}", "OFFSET {offset}"),
            (Some(_), None) => ("LIMIT {limit}", ""),
            (None, Some(_)) => ("LIMIT -1", "OFFSET {offset}"),
            (None, None) => ("", ""),
        },
    )
    .fetch_all(&pool)
    .await?;

    for result in results {
        println!(
            "{id}: {name} - {description}",
            id = result.id,
            name = result.name,
            description = result.description
        );
    }

    Ok(())
}
