use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
    pub host: String,
    pub port: u16,
    pub rate_limit_per_minute: u32,
}

impl Config {
    pub fn from_env() -> Self {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/code_share".to_string());
        
        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your_super_secret_jwt_key_here".to_string());
        
        let jwt_expiry_hours = env::var("JWT_EXPIRY_HOURS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(24);
        
        let host = env::var("HOST")
            .unwrap_or_else(|_| "0.0.0.0".to_string());
        
        let port = env::var("PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8080);
        
        let rate_limit_per_minute = env::var("RATE_LIMIT_PER_MINUTE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60);

        Config {
            database_url,
            jwt_secret,
            jwt_expiry_hours,
            host,
            port,
            rate_limit_per_minute,
        }
    }
}
