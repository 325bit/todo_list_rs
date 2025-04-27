use dioxus::prelude::*;
#[cfg(feature = "server")]
use server_utils::{get_db_pool, sqlx_err_to_server_err};
#[cfg(feature = "server")]
use sqlx::Row;
#[cfg(feature = "server")]
use std::env;

#[server]
pub async fn save_todo(content: String) -> Result<(), ServerFnError> {
    if content.trim().is_empty() {
        return Err(ServerFnError::ServerError(
            "Todo content cannot be empty".to_string(),
        ));
    }

    // Get the pool (initializes on first call across the application)
    let pool = get_db_pool().await.map_err(sqlx_err_to_server_err)?;

    println!("Executing INSERT query for: {}", content);

    // Execute the query
    sqlx::query("INSERT INTO todos (content) VALUES ($1)")
        .bind(content)
        .execute(pool)
        .await
        .map_err(sqlx_err_to_server_err)?;

    println!("INSERT query successful.");
    Ok(())
}

#[server]
pub async fn list_todos() -> Result<Vec<(usize, String)>, ServerFnError> {
    let pool = get_db_pool().await.map_err(sqlx_err_to_server_err)?;

    println!("Executing SELECT query for todos.");

    let rows = sqlx::query(
        r#"
        SELECT id, content
        FROM todos
        ORDER BY id DESC
        LIMIT 10
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(sqlx_err_to_server_err)?;

    println!("SELECT query successful. Found {} rows.", rows.len());

    // Process rows - Use map and collect with error propagation
    let todos = rows
        .into_iter()
        .map(|row| {
            // Use try_get for potentially fallible conversions
            let id: i32 = row.try_get("id")?;
            let content: String = row.try_get("content")?;
            Ok((id as usize, content))
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(sqlx_err_to_server_err)?;

    Ok(todos)
}
#[server]
pub async fn delete_todo(id: usize) -> Result<(), ServerFnError> {
    let pool = get_db_pool().await.map_err(sqlx_err_to_server_err)?;

    println!("Executing DELETE query for id: {}", id);

    // SQL query to delete by ID
    sqlx::query("DELETE FROM todos WHERE id = $1")
        .bind(id as i32) // Cast usize back to i32 for SQL
        .execute(pool)
        .await
        .map_err(sqlx_err_to_server_err)?;

    println!("DELETE query successful for id: {}", id);
    Ok(())
}
#[cfg(feature = "server")]
pub mod server_utils {

    use dioxus::prelude::*;
    use sqlx::{postgres::PgPoolOptions, PgPool};
    use std::env;
    use tokio::sync::OnceCell;

    static DB_POOL: OnceCell<PgPool> = OnceCell::const_new();

    pub async fn get_db_pool() -> Result<&'static PgPool, sqlx::Error> {
        DB_POOL
            .get_or_try_init(|| async {
                let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
                    "postgresql://postgres:123@localhost:5432/todo_list".to_string()
                });

                println!("Initializing database pool with URL: {}", database_url);

                let pool = PgPoolOptions::new()
                    .max_connections(10)
                    .connect(&database_url)
                    .await?;

                println!("Database pool connected. Running migrations...");

                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS todos (
                    id SERIAL PRIMARY KEY,
                    content TEXT NOT NULL
                );",
                )
                .execute(&pool)
                .await?;

                println!("Migrations complete. Database pool initialized successfully.");
                Ok(pool)
            })
            .await
    }

    pub fn sqlx_err_to_server_err(e: sqlx::Error) -> ServerFnError {
        ServerFnError::ServerError(format!("Database error: {}", e))
    }
}
