use crate::{auth_and_login,utilities};
use axum::{
    body::{Bytes, Full},
    extract::{Form, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct Item {
    name: String,
    url: String,
    price: f32,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteRequest {
    id: u32,
}

#[derive(Serialize, Deserialize)]
pub struct RegistrationRequest {
    pub username: String,
    pub password: String,
}

// Serve a static file
pub async fn load_file(Path(file_name): Path<String>) -> Response<Full<Bytes>> {
    let file_path = format!("./assets/{}", file_name);
    let parts: Vec<&str> = file_name.split('.').collect();
    let file_extension = parts[parts.len() - 1].to_lowercase();
    let mime_type = match file_extension.as_str() {
        "html" => "text/html",
        "js" => "text/javascript",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "css" => "text/css",
        _default => "text/html",
    };

    let mut status_code = StatusCode::OK;
    let body: Vec<u8>;
    match std::path::Path::new(&file_path).exists() {
        true => body = fs::read(&file_path).unwrap(),
        false => {
            body = fs::read("./assets/404.html").unwrap();
            status_code = StatusCode::NOT_FOUND;
        }
    }
    Response::builder()
        .status(status_code)
        .header("Content-Type", mime_type)
        .body(Full::from(body))
        .unwrap()
}

pub async fn process_login(
    State(state): State<SqlitePool>,
    form_data: Form<auth_and_login::LoginRequest>,
) -> (HeaderMap, Html<String>) {
    let mut headers = HeaderMap::new();
    let response_html;
    if auth_and_login::verify_login(
        form_data.username.as_str(),
        form_data.password.as_str(),
        state.clone(),
    )
    .await
    {
        response_html = "".to_string();

        let token = auth_and_login::generate_token();
        let cookie_duration_in_seconds: i64 = 3600000;
        let current_time: i64 = utilities::get_epoch_time();
        let expiry: i64 = current_time + cookie_duration_in_seconds;

        sqlx::query("INSERT INTO auth_tokens (token,user_id,expiry,revoked) values(?,?,?,?)")
            .bind(&token)
            .bind(1)
            .bind(expiry)
            .bind(false)
            .execute(&state)
            .await
            .expect("Failed to create access token");

        headers.insert(
            "Set-Cookie",
            format!("auth_token={}; Max-Age={}; HttpOnly", &token, expiry)
                .parse()
                .unwrap(),
        );
        headers.insert("HX-Location", "./home.html".parse().unwrap());
    } else {
        response_html = "Invalid username or password".to_string();

        headers.insert("HX-Retarget", "#login-response".parse().unwrap());
    }
    (headers, Html(response_html))
}

pub async fn register(
    State(state): State<SqlitePool>,
    Form(form_data): Form<RegistrationRequest>,
)  {
    sqlx::query("INSERT INTO users(username,hashed_password) values(?,?)")
        .bind(form_data.username)
        .bind(auth_and_login::hash_password(form_data.password))
        .execute(&state).await.expect("Failed to create registration");
}

pub async fn add_item(
    State(state): State<SqlitePool>,
    Form(form_data): Form<Item>,
) -> Html<String> {
    sqlx::query("INSERT INTO lists (user_id,name,url,price,taken) values(1234,?,?,?,false)")
        .bind(&form_data.name)
        .bind(&form_data.url)
        .bind(utilities::format_currency(form_data.price))
        .execute(&state)
        .await
        .expect("Failed to add item to list.");

    Html(format!(
        "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
        form_data.name,
        form_data.url,
        utilities::format_currency(form_data.price)
    ))
}

pub async fn delete_item(delete_request: Query<DeleteRequest>) {
    println!("Trying to remove item {}", delete_request.id);
}
