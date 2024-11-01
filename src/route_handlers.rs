use crate::auth_and_login::User;
use crate::{auth_and_login, utilities, AppState};
use axum::{
    extract::{Form, Path, State},
    http::{HeaderMap, StatusCode},
    response::Html,
};
use futures::TryStreamExt;
use html_escape::encode_text;
use serde::{Deserialize, Serialize};
use sqlx::Row;

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
    pub email: String,
    pub username: String,
    pub password: String,
    pub confirm_password: String,
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
    pub item_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct UpdatePasswordRequest {
    pub current_password: String,
    pub password: String,
    pub confirm_password: String,
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

// pub async fn register(State(state): State<AppState>, Form(form_data): Form<RegistrationRequest>) {
//     if form_data.password != form_data.confirm_password {
//         panic!("Passwords do not match");
//     }
//     sqlx::query("INSERT INTO users(email,username,hashed_password) values(?,?,?)")
//         .bind(form_data.email)
//         .bind(form_data.username)
//         .bind(auth_and_login::hash_password(form_data.password))
//         .execute(&state.connection_pool)
//         .await
//         .expect("Failed to create registration");
// }

pub async fn add_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form_data): Form<Item>,
) -> (HeaderMap, Html<String>) {
    let user_id = utilities::get_user_id_from_header(headers);
    let mut response_headers = HeaderMap::new();

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

    response_headers.insert("HX-Trigger-After-Swap", "somePresents".parse().unwrap());

    let created_id: i32 = new_row.try_get("id").unwrap();

    (response_headers,Html(format!("<tr><td><a href='{}'>{}</a></td><td>{}</td><td style='text-align:center'><i class='fa-regular fa-x'></i></td><td><a href='#' hx-delete='./item/{}' hx-target='closest tr' hx-swap='outerHTML' hx-confirm='Please confirm you wish to delete {} from your list'><i class=\"fa-duotone fa-trash-can\"></i></a></td></tr>\n",
                form_data.url, form_data.name, utilities::format_currency(form_data.price),created_id,form_data.name)))
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
) -> (HeaderMap, Html<String>) {
    let mut response_headers = HeaderMap::new();

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
        response_headers.insert("HX-Trigger", "showAddForm".parse().unwrap());
        res.push_str(
            "<thead><th>Name</th><th>Price</th><th>Taken</th><th>Delete</th></tr></thead>\n<tbody>",
        );
    } else {
        response_headers.insert("HX-Trigger", "hideAddForm".parse().unwrap());
        res.push_str(
            "<thead><th>Name</th><th>Price</th><th>Taken</th><th class='taken-by'>Taken by</th><th>Action</th></tr></thead>\n",
        );
    }
    let mut row_count = 0;
    while let Some(row) = presents.try_next().await.unwrap() {
        row_count += 1;
        let taken: String = if row.taken {
            "<i class='fa-regular fa-check'></i>".to_string()
        } else {
            "<i class='fa-regular fa-x'></i>".to_string()
        };
        if user_id == requested_user_id {
            res = format!(
                "{}<tr><td><a href='{}'>{}</a></td><td>{}</td><td style='text-align:center'>{}</td><td><a href='#' hx-target='closest tr' hx-swap='outerHTML' hx-delete='./item/{}' hx-confirm='Please confirm you wish to delete {} from your list'><i class=\"fa-duotone fa-trash-can\"></i></a></td></tr>\n",
                res, encode_text(&row.url), encode_text(&row.name), encode_text(&row.price), taken,row.id,encode_text(&row.name)
            );
        } else {
            let buying_it_text: String = if row.taken {
                "".to_string()
            } else {
                "<i class='fa-sharp fa-solid fa-cart-plus'></i>".to_string()
            };
            res = format!(
                "{}<tr><td><a href='{}'>{}</a></td><td>{}</td><td style='text-align:center'>{}</td><td class='taken-by'>{}</td><td><a hx-patch='./item/{}' hx-confirm='Please confirm you are buying or have bought {}' hx-target='closest tr' href='#'>{}</a></td></tr>\n",
                res, encode_text(&row.url), encode_text(&row.name), encode_text(&row.price), taken, encode_text(&row.taken_by_name.unwrap_or_default()),row.id,encode_text(&row.name), buying_it_text
            );
        }
    }
    res.push_str("</tbody></table>");
    if row_count == 0 {
        response_headers.insert("HX-Trigger-After-Swap", "noPresents".parse().unwrap());
        if user_id != requested_user_id {
            res.push_str("<p class='no-presents'>This person's list is currently empty.</p>");
        } else {
            res.push_str(
                "<p class='no-presents'>You have no items in your list, try adding some below.</p>",
            );
        }
    } else {
        response_headers.insert("HX-Trigger-After-Swap", "somePresents".parse().unwrap());
    }
    (response_headers, Html(res))
}

pub async fn get_users(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let calling_user: User = auth_and_login::get_user(
        utilities::get_user_id_from_header(headers),
        state.connection_pool.to_owned(),
    )
    .await
    .expect("Unable to fetch user");

    let mut users_list = format!(
        "<select hx-target='#items' hx-get='./items/' hx-on='htmx:configRequest: event.detail.path += this.value' id='users-list' name='users-list'><option value='{}'>Your list</option>",
        calling_user.id
    );
    let mut rows = sqlx::query("SELECT username,id FROM users ORDER by username ASC")
        .fetch(&state.connection_pool);

    while let Some(row) = rows.try_next().await.unwrap() {
        let username: &str = row.try_get("username").unwrap();
        let user_id: i32 = row.try_get("id").unwrap();
        if user_id == calling_user.id {
        } else {
            users_list = format!(
                "{}<option value='{}'>{}</option>",
                users_list, user_id, username
            );
        }
    }

    users_list.push_str("</select>");

    Html(users_list)
}

pub async fn allocate_item(
    State(state): State<AppState>,
    allocated_item: Path<AllocateItemRequest>,
    headers: HeaderMap,
) -> Html<String> {
    let user_id = utilities::get_user_id_from_header(headers);
    let user = auth_and_login::get_user(user_id, state.connection_pool.clone())
        .await
        .expect("Unable to get username");
    let result = sqlx::query(
        "UPDATE presents SET taken=true, taken_by_id=? WHERE id=? RETURNING name,url,price",
    )
    .bind(user_id)
    .bind(allocated_item.item_id)
    .fetch_one(&state.connection_pool)
    .await
    .expect("Failed to allocate item.");

    let url: String = result.try_get("url").unwrap();
    let name: String = result.try_get("name").unwrap();
    let price: String = result.try_get("price").unwrap();

    Html(format!(
        "<td><a href='{}'>{}</a></td><td>{}</td><td><i class='fa-regular fa-check'></i></td><td>{}</td><td></td>",
        url, name, price, user.username
    ))
}

pub async fn update_password(
    State(_state): State<AppState>,
    _headers: HeaderMap,
    Form(request): Form<UpdatePasswordRequest>,
) -> (StatusCode, Html<String>) {
    let password = request.password;
    let confirm_password = request.confirm_password;
    if password != confirm_password {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Html("Your passwords must match".to_string()),
        );
    }
    let _existing_password = request.current_password;
    (StatusCode::OK, Html("".to_string()))
}
