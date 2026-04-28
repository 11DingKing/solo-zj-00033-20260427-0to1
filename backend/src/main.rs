pub mod config;
pub mod models;
pub mod handlers;
pub mod middleware;
pub mod utils;
pub mod db;

use actix_web::{App, HttpServer, web, middleware::Logger};
use actix_cors::Cors;
use dotenv::dotenv;
use env_logger::Env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let config = config::Config::from_env();
    let pool = db::create_pool(&config.database_url).await;

    db::run_migrations(&pool).await.expect("Failed to run migrations");

    let host = config.host.clone();
    let port = config.port;

    println!("Starting server on {}:{}", host, port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .wrap(Logger::default())
            .wrap(cors)
            .wrap(middleware::rate_limit::RateLimitMiddleware)
            .configure(handlers::auth::config)
            .configure(handlers::users::config)
            .configure(handlers::snippets::config)
            .configure(handlers::search::config)
            .configure(handlers::embed::config)
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
