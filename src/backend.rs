use dioxus::prelude::*;

#[cfg(feature = "server")]
use server_utils::{get_db_pool, sqlx_err_to_server_err};
#[cfg(feature = "server")]
use sqlx::{postgres::PgRow, Row}; // Import PgRow for row access
#[cfg(feature = "server")]
use std::env;

// --- New Server Functions for Auth ---

#[server]
pub async fn register(username: String, password: String) -> Result<(), ServerFnError> {
    // Basic validation
    if username.trim().is_empty() || password.trim().is_empty() {
        return Err(ServerFnError::ServerError(
            "Username and password cannot be empty".to_string(),
        ));
    }

    let pool = get_db_pool().await.map_err(sqlx_err_to_server_err)?;

    // Check if username already exists
    let user_exists: Option<i32> = sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
        .bind(&username)
        .fetch_optional(&*pool)
        .await
        .map_err(sqlx_err_to_server_err)?;

    if user_exists.is_some() {
        return Err(ServerFnError::ServerError(
            "Username already taken".to_string(),
        ));
    }

    // Insert the new user (password stored as plain text per requirement)
    sqlx::query("INSERT INTO users (username, password) VALUES ($1, $2)")
        .bind(&username)
        .bind(&password) // Storing password as plain text
        .execute(&*pool)
        .await
        .map_err(sqlx_err_to_server_err)?;

    println!("User registered: {}", username);
    Ok(())
}

#[server]
pub async fn login(username: String, password: String) -> Result<(), ServerFnError> {
    // Basic validation
    if username.trim().is_empty() || password.trim().is_empty() {
        return Err(ServerFnError::ServerError(
            "Username and password cannot be empty".to_string(),
        ));
    }

    let pool = get_db_pool().await.map_err(sqlx_err_to_server_err)?;

    // Retrieve user by username
    let row: Option<PgRow> = sqlx::query("SELECT id, password FROM users WHERE username = $1")
        .bind(&username)
        .fetch_optional(&*pool)
        .await
        .map_err(sqlx_err_to_server_err)?;

    match row {
        Some(row) => {
            let stored_password: String =
                row.try_get("password").map_err(sqlx_err_to_server_err)?;
            // Compare provided password with stored password (plain text comparison)
            if stored_password == password {
                println!("User logged in: {}", username);
                Ok(()) // Login successful
            } else {
                println!("Login failed for {}: Invalid password", username);
                Err(ServerFnError::ServerError(
                    "Invalid username or password".to_string(),
                ))
            }
        }
        None => {
            println!("Login failed: User {} not found", username);
            Err(ServerFnError::ServerError(
                "Invalid username or password".to_string(),
            ))
        }
    }
}

// --- Modified Todo Server Functions (now require username) ---

// Helper to get user ID from username
#[cfg(feature = "server")]
async fn get_user_id(pool: &sqlx::PgPool, username: &str) -> Result<i32, ServerFnError> {
    let user_row: Option<PgRow> = sqlx::query("SELECT id FROM users WHERE username = $1")
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(sqlx_err_to_server_err)?;

    match user_row {
        Some(row) => row.try_get("id").map_err(sqlx_err_to_server_err),
        None => Err(ServerFnError::ServerError("User not found".to_string())),
    }
}

#[server]
pub async fn save_todo(username: String, content: String) -> Result<(), ServerFnError> {
    if content.trim().is_empty() {
        return Err(ServerFnError::ServerError(
            "Todo content cannot be empty".to_string(),
        ));
    }

    let pool = get_db_pool().await.map_err(sqlx_err_to_server_err)?;

    // Get the user_id for the given username
    let user_id = get_user_id(&*pool, &username).await?;

    println!(
        "Executing INSERT query for: {} for user {}",
        content, username
    );

    // Execute the query, inserting user_id as well
    sqlx::query("INSERT INTO todos (user_id, content) VALUES ($1, $2)")
        .bind(user_id) // Bind user_id
        .bind(content)
        .execute(&*pool)
        .await
        .map_err(sqlx_err_to_server_err)?;

    println!("INSERT query successful for user {}", username);
    Ok(())
}

#[server]
pub async fn list_todos(username: String) -> Result<Vec<(usize, String)>, ServerFnError> {
    let pool = get_db_pool().await.map_err(sqlx_err_to_server_err)?;

    // Get the user_id for the given username
    let user_id = get_user_id(&*pool, &username).await?;

    println!("Executing SELECT query for todos for user {}.", username);

    // Select only todos belonging to this user_id
    let rows = sqlx::query(
        r#"
        SELECT id, content
        FROM todos
        WHERE user_id = $1
        ORDER BY id DESC
        LIMIT 10
        "#,
    )
    .bind(user_id) // Bind user_id to filter
    .fetch_all(&*pool)
    .await
    .map_err(sqlx_err_to_server_err)?;

    println!(
        "SELECT query successful for user {}. Found {} rows.",
        username,
        rows.len()
    );

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
pub async fn delete_todo(username: String, id: usize) -> Result<(), ServerFnError> {
    let pool = get_db_pool().await.map_err(sqlx_err_to_server_err)?;

    // Get the user_id for the given username
    let user_id = get_user_id(&*pool, &username).await?;

    println!(
        "Executing DELETE query for todo id: {} for user: {}",
        id, username
    );

    // SQL query to delete by ID *and* user_id
    // This prevents a user from deleting another user's todo
    sqlx::query("DELETE FROM todos WHERE id = $1 AND user_id = $2")
        .bind(id as i32) // Cast usize back to i32 for SQL id
        .bind(user_id) // Bind user_id
        .execute(&*pool)
        .await
        .map_err(sqlx_err_to_server_err)?;

    println!(
        "DELETE query successful for todo id: {} for user: {}",
        id, username
    );
    Ok(())
}

#[cfg(feature = "server")]
pub mod server_utils {
    use dioxus::prelude::*;
    use sqlx::{postgres::PgPoolOptions, PgPool};
    use std::env;
    use tokio::sync::OnceCell;

    static DB_POOL: OnceCell<PgPool> = OnceCell::const_new();

    // Modified get_db_pool to initialize both users and todos tables
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

                // Create users table if it doesn't exist
                let create_users_table = r#"
                    CREATE TABLE IF NOT EXISTS users (
                        id SERIAL PRIMARY KEY,
                        username TEXT UNIQUE NOT NULL,
                        password TEXT NOT NULL
                    );
                "#;
                sqlx::query(create_users_table).execute(&pool).await?;
                println!("'users' table migration complete.");

                // Create todos table if it doesn't exist (now with user_id)
                let create_todos_table = r#"
                    CREATE TABLE IF NOT EXISTS todos (
                        id SERIAL PRIMARY KEY,
                        user_id INTEGER NOT NULL,
                        content TEXT NOT NULL,
                        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
                    );
                "#;
                sqlx::query(create_todos_table).execute(&pool).await?;
                println!("'todos' table migration complete (with user_id).");

                println!("Database pool initialized successfully.");
                Ok(pool)
            })
            .await
    }

    pub fn sqlx_err_to_server_err(e: sqlx::Error) -> ServerFnError {
        // Log the detailed SQLx error on the server side
        eprintln!("SQLx error: {:?}", e);
        // Return a more generic error to the client for security/simplicity
        ServerFnError::ServerError("A database error occurred.".to_string())
        // Or you could choose to expose more details depending on your needs:
        // ServerFnError::ServerError(format!("Database error: {}", e))
    }
}
