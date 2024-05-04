use std::net::SocketAddr;

use std::env;

pub struct AppConfig<'a> {
    pub addr: SocketAddr,
    pub file_path: &'a str,
    pub use_tls: bool,
}

pub fn get_app_config<'a>() -> AppConfig<'a> {
    let environment = env::var("APP_ENVIRONMENT");

    match environment {
        Ok(i) => match i.as_str() {
            "PRODUCTION" => AppConfig {
                addr: SocketAddr::from(([0, 0, 0, 0], 443)),
                file_path: "/app/christmas_lists/assets",
                use_tls: true,
            },
            "TEST" => AppConfig {
                addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
                file_path: "/assets",
                use_tls: true,
            },
            _ => {
                println!("Please set APP_ENVIRONMENT variable to either PRODUCTION or TEST");
                AppConfig {
                    addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
                    file_path: "/assets",
                    use_tls: true,
                }
            }
        },
        Err(_e) => {
            println!("Please set APP_ENVIRONMENT variable to either PRODUCTION or TEST");
            AppConfig {
                addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
                file_path: "/assets",
                use_tls: true,
            }
        }
    }
}
