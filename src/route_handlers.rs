use crate::{auth_and_login, utilities, AppState};
use axum::{
    body::{Bytes, Full},
    extract::{Form, Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, Response},
};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct Item {
    name: String,
    url: String,
    price: f32,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteRequest {
    item_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct GetItemsRequest {
    user_id: i32,
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
    State(state): State<AppState>,
    form_data: Form<auth_and_login::LoginRequest>,
) -> (HeaderMap, Html<String>) {
    let mut headers = HeaderMap::new();
    let response_html;
    match auth_and_login::verify_login(
        form_data.username.as_str(),
        form_data.password.as_str(),
        state.connection_pool.clone(),
    )
    .await
    {
        Some(_value) => {
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
                .execute(&state.connection_pool)
                .await
                .expect("Failed to create access token");

            headers.insert(
                "Set-Cookie",
                format!("auth_token={}; Max-Age={}; HttpOnly", &token, expiry)
                    .parse()
                    .unwrap(),
            );
            headers.insert("HX-Location", "./home.html".parse().unwrap());
        }
        None => {
            response_html = "Invalid username or password".to_string();

            headers.insert("HX-Retarget", "#login-response".parse().unwrap());
        }
    }
    (headers, Html(response_html))
}

pub async fn register(State(state): State<AppState>, Form(form_data): Form<RegistrationRequest>) {
    sqlx::query("INSERT INTO users(username,hashed_password) values(?,?)")
        .bind(form_data.username)
        .bind(auth_and_login::hash_password(form_data.password))
        .execute(&state.connection_pool)
        .await
        .expect("Failed to create registration");
}

pub async fn add_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form_data): Form<Item>,
) -> Html<String> {
    let user_id = utilities::get_user_id_from_header(headers);

    sqlx::query("INSERT INTO presents (user_id,name,url,price,taken) values(?,?,?,?,false)")
        .bind(user_id)
        .bind(&form_data.name)
        .bind(&form_data.url)
        .bind(utilities::format_currency(form_data.price))
        .execute(&state.connection_pool)
        .await
        .expect("Failed to add item to list.");

    Html(format!(
        "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
        form_data.name,
        form_data.url,
        utilities::format_currency(form_data.price)
    ))
}

pub async fn delete_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    delete_request: Path<DeleteRequest>,
) -> Html<String> {
    let user_id = utilities::get_user_id_from_header(headers);
    sqlx::query("DELETE FROM presents WHERE id=? AND user_id=?")
        .bind(delete_request.item_id)
        .bind(user_id)
        .execute(&state.connection_pool)
        .await
        .expect("Failed to delete item from list.");

    Html("Deleted item from list.".to_string())
}

pub async fn get_items(
    State(state): State<AppState>,
    headers: HeaderMap,
    items_request: Path<GetItemsRequest>,
) -> Html<&'static str> {
    let user_id = utilities::get_user_id_from_header(headers);
    let requested_user_id = items_request.user_id;
    if user_id == requested_user_id {
        Html("You are requesting your own list")
    } else {
        Html("You are requesting someone else's list")
    }
}

pub async fn get_users(State(state): State<AppState>) -> Html<String> {
    let mut users_list = String::new();
    let mut rows = sqlx::query("SELECT username,id FROM users").fetch(&state.connection_pool);

    while let Some(row) = rows.try_next().await.unwrap() {
        let username: &str = row.try_get("username").unwrap();
        let user_id: i32 = row.try_get("id").unwrap();
        users_list = format!(
            "{}<option value='{}'>{}</option>",
            users_list, user_id, username
        );
    }

    Html(users_list)
}

pub async fn allocate_item() {
    println!("Trying to allocate item");
}
