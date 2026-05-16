use axum::http::{HeaderValue, Method};
use std::env;
use tower_http::cors::CorsLayer;

pub fn create_cors_layer() -> CorsLayer {
    let is_dev = env::var("APP_ENV").unwrap_or_else(|_| "production".to_string()) == "development";

    if is_dev {
        // Allow all origins in development
        CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
            .allow_headers(tower_http::cors::Any)
            .allow_credentials(false)
    } else {
        // Production configuration
        CorsLayer::new()
            .allow_origin([
                "https://a350.innosolve.in".parse::<HeaderValue>().unwrap(),
                "http://a350.innosolve.in".parse::<HeaderValue>().unwrap(),
                "https://srm-leetcode.vercel.app".parse::<HeaderValue>().unwrap(),
                "http://srm-leetcode.vercel.app".parse::<HeaderValue>().unwrap(),
//                "http://localhost:3000".parse::<HeaderValue>().unwrap(),
//                "https://localhost:3000".parse::<HeaderValue>().unwrap(),
                // Add more origins here as needed
            ])
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
            .allow_headers([
                axum::http::header::CONTENT_TYPE,
                axum::http::header::AUTHORIZATION,
            ])
            .allow_credentials(true)
    }
}
