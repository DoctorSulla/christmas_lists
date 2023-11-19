use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod auth_and_login;
pub mod route_handlers;
pub mod utilities;

#[derive(Clone)]
pub struct AppState {
    connection_pool: SqlitePool,
    user: Arc<Mutex<User>>,
}

#[derive(Clone,sqlx::FromRow)]
pub struct User {
    user_id: i16,
    username: String,
}

#[tokio::main]
async fn main() {
    // Set connection options
    let connection_options = SqliteConnectOptions::new()
        .filename("christmas_lists.db")
        .create_if_missing(true);

    // Create pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connection_options)
        .await
        .expect("Failed to create connection pool");

    let app_state: AppState = AppState {
        connection_pool: pool,
        user: Arc::new(Mutex::new(User {
            user_id: 0,
            username: String::new(),
        }))
    };
    // Create tables

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS auth_tokens (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            token VARCHAR(30),
            user_id INTEGER,
            expiry INTEGER,
            revoked BOOLEAN)",
    )
    .execute(&app_state.connection_pool)
    .await
    .expect("Failed to create table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT, 
            username VARCHAR(30) UNIQUE, 
            hashed_password VARCHAR(200))
        ",
    )
    .execute(&app_state.connection_pool)
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
    .execute(&app_state.connection_pool)
    .await
    .expect("Failed to create table");

    let protected_routes = Router::new()
        .route("/item", post(route_handlers::add_item))
        .route("/item", delete(route_handlers::delete_item))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_and_login::auth,
        ));

    let open_routes = Router::new()
        .route("/assets/:file_name", get(route_handlers::load_file))
        .route("/login", post(route_handlers::process_login))
        .route("/register", post(route_handlers::register));

    let app = Router::new()
        .merge(protected_routes)
        .merge(open_routes)
        .with_state(app_state);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
