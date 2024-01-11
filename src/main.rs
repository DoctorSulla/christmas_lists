use axum::{
    middleware,
    routing::{delete, get, patch, post},
    Router,
};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};

use std::env;

use axum_server;
//use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;

pub mod auth_and_login;
pub mod route_handlers;
pub mod tables;
pub mod utilities;

#[derive(Clone)]
pub struct AppState {
    connection_pool: SqlitePool,
    file_path: String,
}

#[derive(Clone, sqlx::FromRow, Debug)]
pub struct User {
    id: i32,
    username: String,
}

#[tokio::main]
async fn main() {
    let environment = env::var("APP_ENVIRONMENT");

    let addr: SocketAddr;
    let file_path: String;

    match environment {
        Ok(i) => match i.as_str() {
            "PRODUCTION" => {
                addr = SocketAddr::from(([0, 0, 0, 0], 80));
                file_path = "/app/christmas_lists/assets/".to_string();
            }
            "TEST" => {
                addr = SocketAddr::from(([127, 0, 0, 1], 3000));
                file_path = "./assets/".to_string();
            }
            _ => {
                println!("Please set APP_ENVIRONMENT variable to either PRODUCTION or TEST");
                addr = SocketAddr::from(([127, 0, 0, 1], 3000));
                file_path = "./assets/".to_string();
            }
        },
        Err(_e) => {
            println!("Please set APP_ENVIRONMENT variable to either PRODUCTION or TEST");
            addr = SocketAddr::from(([127, 0, 0, 1], 3000));
            file_path = "./assets/".to_string();
        }
    }

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

    // Create tables
    tables::create(pool.clone()).await;

    // Create TLS Config
    // let config = RustlsConfig::from_pem_file("./certs/cert.pem", "./certs/key.pem")
    //     .await
    //     .unwrap();

    let app_state: AppState = AppState {
        connection_pool: pool,
        file_path: file_path.clone(),
    };

    let protected_routes = Router::new()
        .route("/item", post(route_handlers::add_item))
        .route("/item/:item_id", delete(route_handlers::delete_item))
        .route("/item/:item_id", patch(route_handlers::allocate_item))
        .route("/items/:user_id", get(route_handlers::get_items))
        .route("/items/", get(route_handlers::get_items))
        .route("/users", get(route_handlers::get_users))
        .route_layer(middleware::map_request_with_state(
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

    //     let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    //     axum_server::bind_rustls(addr, config)
    //         .serve(app.into_make_service())
    //         .await
    //         .unwrap();

    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
