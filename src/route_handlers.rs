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
    user_id: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct RegistrationRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Present {
    id: i32,
    name: String,
    url: String,
    price: String,
    taken: bool,
    #[sqlx(rename = "username")]
    taken_by_name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AllocateItemRequest {
    pub id: i32,
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
        Some(value) => {
            response_html = "".to_string();

            let token = auth_and_login::generate_token();
            let cookie_duration_in_seconds: i64 = 3600000;
            let current_time: i64 = utilities::get_epoch_time();
            let expiry: i64 = current_time + cookie_duration_in_seconds;

            sqlx::query("INSERT INTO auth_tokens (token,user_id,expiry,revoked) values(?,?,?,?)")
                .bind(&token)
                .bind(value.id)
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

    let new_row = sqlx::query(
        "INSERT INTO presents (user_id,name,url,price,taken) values(?,?,?,?,false) RETURNING id",
    )
    .bind(user_id)
    .bind(&form_data.name)
    .bind(&form_data.url)
    .bind(utilities::format_currency(form_data.price))
    .fetch_one(&state.connection_pool)
    .await
    .expect("Failed to add item to list.");

    let created_id: i32 = new_row.try_get("id").unwrap();

    Html(format!("<tr><td><a href='{}'>{}</a></td><td>{}</td><td>{}</td><td><a href='#' hx-delete='../item/{}' hx-target='closest tr' hx-swap='outerHTML' hx-confirm='Please confirm you wish to delete {} from your list'>&times;</a></td></tr>\n",
                form_data.url, form_data.name, utilities::format_currency(form_data.price),false,created_id,form_data.name))
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

    Html("".to_string())
}

pub async fn get_items(
    State(state): State<AppState>,
    headers: HeaderMap,
    items_request: Path<GetItemsRequest>,
) -> Html<String> {
    let user_id = utilities::get_user_id_from_header(headers);
    let requested_user_id = match items_request.user_id {
        Some(i) => i,
        None => user_id,
    };
    let mut presents = sqlx::query_as::<_, Present>(
        "SELECT 
            p.id,
            p.name,
            p.url,
            p.price,
            p.taken,
            u.username
        FROM 
            presents p 
        LEFT JOIN 
            users u 
        ON 
            p.taken_by_id = u.id 
        WHERE 
            user_id=?",
    )
    .bind(requested_user_id)
    .fetch(&state.connection_pool);

    let mut res = String::from("<table id='list-table'>");
    if user_id == requested_user_id {
        res.push_str(
            "<thead><th>Name</th><th>Price</th><th>Allocated</th><th>Delete</th></tr></thead>\n<tbody>",
        );
    } else {
        res.push_str(
            "<thead><th>Name</th><th>Price</th><th>Allocated</th><th>Allocated to</th><th>Allocate</th></tr></thead>\n",
        );
    }
    while let Some(row) = presents.try_next().await.unwrap() {
        if user_id == requested_user_id {
            res = format!(
                "{}<tr><td><a href='{}'>{}</a></td><td>{}</td><td>{}</td><td><a href='#' hx-target='closest tr' hx-swap='outerHTML' hx-delete='../item/{}' hx-confirm='Please confirm you wish to delete {} from your list'>&times;</a></td></tr>\n",
                res, row.url, row.name, row.price, row.taken,row.id,row.name
            );
        } else {
            res = format!(
                "{}<tr><td><a href='{}'>{}</a></td><td>{}</td><td>{}</td><td>{}</td><td><a hx-patch='../item/{}' hx-confirm='Please confirm you are buying or have bought {}' href='#'>I'm buying this</a></td></tr>\n",
                res, row.url, row.name, row.price, row.taken, row.taken_by_name.unwrap_or_default(),row.id,row.name
            );
        }
    }
    res.push_str("</tbody></table>");
    Html(res)
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

pub async fn allocate_item(
    State(state): State<AppState>,
    allocated_item: Path<AllocateItemRequest>,
    headers: HeaderMap,
) -> Html<String> {
    println!("Trying to allocate item");
    let user_id = utilities::get_user_id_from_header(headers);

    sqlx::query("UPDATE presents SET taken=true, taken_by_id=? WHERE id=?")
        .bind(user_id)
        .bind(allocated_item.id)
        .execute(&state.connection_pool)
        .await
        .expect("Failed to allocate item.");

    Html("Successfully allocated item.".to_string())
}
