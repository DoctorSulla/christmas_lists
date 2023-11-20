use crate::{utilities, AppState, User};
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    TypedHeader,
};
use headers::Cookie;
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

// Verify the username and password match stored credentials and return the user if so
pub async fn verify_login(username: &str, password: &str, pool: SqlitePool) -> Option<User> {
    // Get hashed password from the database
    let query = match sqlx::query("SELECT id,hashed_password FROM users WHERE username=?")
        .bind(username)
        .fetch_optional(&pool)
        .await {
            Ok(result) => match result {
                Some(row) => row,
                None => return None
            },
            Err(_e) => return None    

        };

    let hashed_password = query.try_get("hashed_password").unwrap();
    let user_id: i32 = query.try_get("id").unwrap();

    // Parse hash
    let parsed_hash = PasswordHash::new(hashed_password).unwrap();

    // Return true if hash matches and false otherwise
    if Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
    {
        get_user(user_id, pool.clone()).await
    } else {
        None
    }
}

// Confirm the cookie is valid and return a the user if so
pub async fn validate_cookie(cookie_value: String, pool: SqlitePool) -> Option<User> {
    let current_time: i64 = utilities::get_epoch_time();

    let query = match sqlx::query(
        "SELECT * FROM auth_tokens WHERE token =? AND expiry < ? and revoked=false",
    )
    .bind(cookie_value)
    .bind(current_time)
    .fetch_optional(&pool)
    .await
    {
        Ok(value) => value,
        Err(_e) => None,
    }
    .unwrap();
    let user_id: i32 = query.try_get("user_id").unwrap();
    get_user(user_id, pool.clone()).await
}

// Middleware to check the auth_token cookie
pub async fn auth<B>(
    State(state): State<AppState>,
    TypedHeader(cookie): TypedHeader<Cookie>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let auth_cookie = cookie.get("auth_token").unwrap_or("");
    match validate_cookie(auth_cookie.to_string(), state.connection_pool.clone()).await {
        Some(value) => {
            let mut data = state.user.lock().await;
            data.username = value.username;
            data.id = value.id;
            let response = next.run(request).await;
            Ok(response)
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

pub fn generate_token() -> String {
    let char_set: Vec<&str> = vec![
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r",
        "s", "t", "u", "v", "w", "x", "y", "z", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
    ];
    let length = 30;
    let mut token = String::new();
    while token.len() <= length {
        let mut rng = rand_chacha::ChaCha20Rng::from_entropy();
        token.push_str(char_set[rng.gen_range(0..char_set.len())]);
    }
    token
}

pub fn hash_password(password: String) -> String {
    let salt = SaltString::generate(rand_chacha::ChaCha20Rng::from_entropy());
    //let salt = SaltString::from_b64("bBYj4SR3zS3fSA+o5yGW6w").unwrap();

    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    password_hash
}

pub async fn get_user(user_id: i32, pool: SqlitePool) -> Option<User> {
    match sqlx::query_as::<_, User>("SELECT id,username FROM users WHERE id=?")
        .bind(user_id)
        .fetch_one(&pool)
        .await
    {
        Ok(value) => Some(value),
        Err(_e) => None,
    }
}
