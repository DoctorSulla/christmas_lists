use crate::{route_handlers, AppState};
use axum::routing::{delete, get, patch, post};
use axum::Router;

pub fn get_protected_routes() -> Router<AppState> {
    Router::new()
        .route("/item", post(route_handlers::add_item))
        .route("/item/:item_id", delete(route_handlers::delete_item))
        .route("/item/:item_id", patch(route_handlers::allocate_item))
        .route("/items/:user_id", get(route_handlers::get_items))
        .route("/items/", get(route_handlers::get_items))
        .route("/password", patch(route_handlers::update_password))
        .route("/users", get(route_handlers::get_users))
}

pub fn get_open_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(route_handlers::process_login))
        .route("/register", post(route_handlers::register))
}
