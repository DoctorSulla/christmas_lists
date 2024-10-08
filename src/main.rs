use anyhow::Error;
use axum::{middleware, Router};
use lettre::{
    transport::smtp::{
        authentication::{Credentials, Mechanism},
        PoolConfig,
    },
    Message, SmtpTransport, Transport,
};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};

use tower_http::services::{ServeDir, ServeFile};

pub mod auth_and_login;
pub mod config;
pub mod route_handlers;
pub mod routes;
pub mod tables;
pub mod utilities;

#[derive(Clone)]
pub struct AppState {
    connection_pool: SqlitePool,
}

fn _send_email(from: &str, to: &str, subject: &str, body: &str) -> Result<(), Error> {
    let email = Message::builder()
        .from(from.parse()?)
        .reply_to(from.parse()?)
        .to(to.parse()?)
        .subject(subject)
        .body(String::from(body))?;

    // Create TLS transport on port 587 with STARTTLS
    let sender = SmtpTransport::starttls_relay("mail.halliday.nz")?
        // Add credentials for authentication
        .credentials(Credentials::new("management".to_owned(), "".to_owned()))
        // Configure expected authentication mechanism
        .authentication(vec![Mechanism::Plain])
        // Connection pool settings
        .pool_config(PoolConfig::new().max_size(20))
        .build();

    // Send the email via remote relay
    let _result = sender.send(&email)?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let app_config: config::AppConfig = config::get_app_config();

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
    let four_o_four = format!("{}/404.html", app_config.file_path);
    let serve_dir =
        ServeDir::new(app_config.file_path).not_found_service(ServeFile::new(four_o_four));

    let app_state: AppState = AppState {
        connection_pool: pool,
    };

    let protected_routes = routes::get_protected_routes().route_layer(
        middleware::map_request_with_state(app_state.clone(), auth_and_login::auth),
    );

    let open_routes = routes::get_open_routes().nest_service("/", serve_dir.clone());

    let app = Router::new()
        .merge(protected_routes)
        .merge(open_routes)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(app_config.addr)
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
