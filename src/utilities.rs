use http::header::HeaderMap;
use num::{Float, FromPrimitive};
use std::time::SystemTime;
// Convert a floating point number to pounds and pence
pub fn format_currency<T>(currency: T) -> String
where
    T: Float,
    T: FromPrimitive,
    T: std::fmt::Display,
{
    let pounds = currency.floor();
    let pence = (currency.fract() * T::from_u8(100).unwrap()).round();
    if pence == T::from_u8(0).unwrap() {
        format!("£{}.00", pounds)
    } else if pence < T::from_u8(10).unwrap() {
        format!("£{}.0{}", pounds, pence)
    } else {
        format!("£{}.{}", pounds, pence)
    }
}

pub fn get_epoch_time() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .try_into()
        .unwrap()
}

pub fn get_user_id_from_header(headers: HeaderMap) -> i32 {
    headers
        .get("user-id")
        .unwrap()
        .to_str()
        .unwrap()
        .parse()
        .unwrap()
}
