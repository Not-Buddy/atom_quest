use axum::http::{HeaderValue, Method};
use std::env;
use tower_http::cors::CorsLayer;

pub fn create_cors_layer() -> CorsLayer {
    let is_dev = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()) == "development";

    if is_dev {
        CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
            .allow_headers(tower_http::cors::Any)
            .allow_credentials(false)
    } else {
        CorsLayer::new()
            .allow_origin([
                "https://atom-quest-seven.vercel.app".parse::<HeaderValue>().unwrap(),
                "http://atom-quest-seven.vercel.app".parse::<HeaderValue>().unwrap(),
            ])
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
            .allow_headers([
                axum::http::header::CONTENT_TYPE,
                axum::http::header::AUTHORIZATION,
            ])
            .allow_credentials(true)
    }
}
