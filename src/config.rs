use std::net::SocketAddr;

use std::env;

pub struct AppConfig<'a> {
    pub addr: SocketAddr,
    pub file_path: &'a str,
}

pub fn get_app_config<'a>() -> AppConfig<'a> {
    let environment = env::var("APP_ENVIRONMENT");

    match environment {
        Ok(i) => match i.as_str() {
            "PRODUCTION" => AppConfig {
                addr: SocketAddr::from(([127, 0, 0, 1], 3003)),
                file_path: "/srv/http/christmaslist.xyz/assets",
            },
            "TEST" => AppConfig {
                addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
                file_path: "./assets",
            },
            _ => {
                println!("Please set APP_ENVIRONMENT variable to either PRODUCTION or TEST");
                AppConfig {
                    addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
                    file_path: "./assets",
                }
            }
        },
        Err(_e) => {
            println!("Please set APP_ENVIRONMENT variable to either PRODUCTION or TEST");
            AppConfig {
                addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
                file_path: "./assets",
            }
        }
    }
}
