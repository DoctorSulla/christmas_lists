
use sqlx::SqlitePool;

// Create tables
pub async fn create(pool: SqlitePool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS auth_tokens (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            token VARCHAR(30),
            user_id INTEGER,
            expiry INTEGER,
            revoked BOOLEAN)",
    )
    .execute(&pool)
    .await
    .expect("Failed to create table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT, 
            username VARCHAR(30) UNIQUE, 
            hashed_password VARCHAR(200))
        ",
    )
    .execute(&pool)
    .await
    .expect("Failed to create table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS lists(
            id INTEGER PRIMARY KEY AUTOINCREMENT, 
            user_id INTEGER,
            name VARCHAR(50),
            url VARCHAR(300), 
            price VARCHAR (30),
            taken BOOLEAN,
            taken_by_id INTEGER
)
        ",
    )
    .execute(&pool)
    .await
    .expect("Failed to create table");
}
