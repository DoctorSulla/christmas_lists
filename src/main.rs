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

use axum_server::tls_rustls::RustlsConfig;

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

    let serve_dir =
        ServeDir::new(app_config.file_path).not_found_service(ServeFile::new("assets/404.html"));

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

    if app_config.use_tls {
        //Create TLS Config
        let tls_config = RustlsConfig::from_pem_file(
            "/etc/christmaslist/live/christmaslist.xyz/fullchain.pem",
            "/etc/christmaslist/live/christmaslist.xyz/privkey.pem",
        )
        .await
        .expect("Failed to create TLS config");

        axum_server::bind_rustls(app_config.addr, tls_config)
            .serve(app.into_make_service())
            .await
            .unwrap();
    } else {
        axum_server::bind(app_config.addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}
