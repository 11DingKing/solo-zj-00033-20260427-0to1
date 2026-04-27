use crate::config::Config;
use crate::models::{User, CreateUser, LoginUser, AuthResponse, UserResponse};
use crate::utils::{jwt, password};
use actix_web::{web, HttpResponse, Responder, Error};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn register(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    form: web::Json<CreateUser>,
) -> Result<impl Responder, Error> {
    let existing_user = sqlx::query!(
        r#"SELECT id FROM users WHERE username = $1 OR email = $2"#,
        form.username,
        form.email
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    if existing_user.is_some() {
        return Ok(HttpResponse::Conflict().json(serde_json::json!({
            "error": "user_exists",
            "message": "Username or email already exists"
        })));
    }

    let password_hash = password::hash_password(&form.password)
        .map_err(|e| {
            eprintln!("Password hash error: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to hash password")
        })?;

    let user_id = Uuid::new_v4();
    
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (id, username, email, password_hash, display_name)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, username, email, password_hash, display_name, avatar_url, created_at, updated_at
        "#,
        user_id,
        form.username,
        form.email,
        password_hash,
        form.display_name
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to create user")
    })?;

    let token = jwt::generate_token(user.id, &user.username, &config)
        .map_err(|e| {
            eprintln!("JWT error: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to generate token")
        })?;

    Ok(HttpResponse::Created().json(AuthResponse {
        token,
        user: UserResponse::from(user),
    }))
}

pub async fn login(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    form: web::Json<LoginUser>,
) -> Result<impl Responder, Error> {
    let user = sqlx::query_as!(
        User,
        r#"SELECT id, username, email, password_hash, display_name, avatar_url, created_at, updated_at FROM users WHERE username = $1"#,
        form.username
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let user = match user {
        Some(u) => u,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "invalid_credentials",
                "message": "Invalid username or password"
            })));
        }
    };

    let valid = password::verify_password(&form.password, &user.password_hash)
        .map_err(|e| {
            eprintln!("Password verify error: {}", e);
            actix_web::error::ErrorInternalServerError("Authentication error")
        })?;

    if !valid {
        return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "invalid_credentials",
            "message": "Invalid username or password"
        })));
    }

    let token = jwt::generate_token(user.id, &user.username, &config)
        .map_err(|e| {
            eprintln!("JWT error: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to generate token")
        })?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user: UserResponse::from(user),
    }))
}

pub async fn get_me(
    pool: web::Data<PgPool>,
    claims: crate::models::Claims,
) -> Result<impl Responder, Error> {
    let user = sqlx::query_as!(
        User,
        r#"SELECT id, username, email, password_hash, display_name, avatar_url, created_at, updated_at FROM users WHERE id = $1"#,
        claims.sub
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    match user {
        Some(u) => Ok(HttpResponse::Ok().json(UserResponse::from(u))),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "user_not_found",
            "message": "User not found"
        }))),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/me", web::get().to(get_me).wrap(crate::middleware::auth::Auth))
    );
}
