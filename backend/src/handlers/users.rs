use crate::models::{User, UserResponse, Claims, SnippetResponse, Snippet, PaginatedResponse};
use actix_web::{web, HttpResponse, Responder, Error};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_user(
    pool: web::Data<PgPool>,
    user_id: web::Path<Uuid>,
) -> Result<impl Responder, Error> {
    let user = sqlx::query_as!(
        User,
        r#"SELECT id, username, email, password_hash, display_name, avatar_url, created_at, updated_at FROM users WHERE id = $1"#,
        user_id.into_inner()
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

pub async fn get_user_snippets(
    pool: web::Data<PgPool>,
    user_id: web::Path<Uuid>,
    query: web::Query<PaginationQuery>,
) -> Result<impl Responder, Error> {
    let user_id = user_id.into_inner();
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let user_exists = sqlx::query!(
        "SELECT id FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    if user_exists.is_none() {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "user_not_found",
            "message": "User not found"
        })));
    }

    let snippets = sqlx::query_as!(
        Snippet,
        r#"
        SELECT id, title, description, language, is_public, user_id, parent_id,
               likes_count, forks_count, views_count, created_at, updated_at
        FROM snippets
        WHERE user_id = $1 AND is_public = true
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        per_page,
        offset
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let total: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) FROM snippets WHERE user_id = $1 AND is_public = true"#,
        user_id
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?
    .unwrap_or(0);

    let mut snippet_responses = Vec::new();
    for snippet in snippets {
        let response = build_snippet_response(&pool, snippet).await?;
        snippet_responses.push(response);
    }

    let total_pages = (total + per_page as i64 - 1) / per_page as i64;

    Ok(HttpResponse::Ok().json(PaginatedResponse {
        data: snippet_responses,
        total,
        page: page as i64,
        per_page: per_page as i64,
        total_pages,
    }))
}

pub async fn get_user_forks(
    pool: web::Data<PgPool>,
    user_id: web::Path<Uuid>,
    query: web::Query<PaginationQuery>,
) -> Result<impl Responder, Error> {
    let user_id = user_id.into_inner();
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let snippets = sqlx::query_as!(
        Snippet,
        r#"
        SELECT s.id, s.title, s.description, s.language, s.is_public, s.user_id, s.parent_id,
               s.likes_count, s.forks_count, s.views_count, s.created_at, s.updated_at
        FROM snippets s
        WHERE s.user_id = $1 AND s.parent_id IS NOT NULL AND s.is_public = true
        ORDER BY s.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        per_page,
        offset
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let total: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) FROM snippets WHERE user_id = $1 AND parent_id IS NOT NULL AND is_public = true"#,
        user_id
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?
    .unwrap_or(0);

    let mut snippet_responses = Vec::new();
    for snippet in snippets {
        let response = build_snippet_response(&pool, snippet).await?;
        snippet_responses.push(response);
    }

    let total_pages = (total + per_page as i64 - 1) / per_page as i64;

    Ok(HttpResponse::Ok().json(PaginatedResponse {
        data: snippet_responses,
        total,
        page: page as i64,
        per_page: per_page as i64,
        total_pages,
    }))
}

async fn build_snippet_response(
    pool: &PgPool,
    snippet: Snippet,
) -> Result<SnippetResponse, Error> {
    let user = sqlx::query_as!(
        User,
        r#"SELECT id, username, email, password_hash, display_name, avatar_url, created_at, updated_at FROM users WHERE id = $1"#,
        snippet.user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let files = sqlx::query_as!(
        crate::models::SnippetFile,
        r#"
        SELECT id, snippet_id, version_id, filename, content, language, created_at
        FROM snippet_files
        WHERE snippet_id = $1
        AND version_id = (
            SELECT id FROM versions WHERE snippet_id = $1 ORDER BY version_number DESC LIMIT 1
        )
        "#,
        snippet.id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let tags: Vec<String> = sqlx::query_scalar!(
        r#"
        SELECT t.name FROM tags t
        JOIN snippet_tags st ON t.id = st.tag_id
        WHERE st.snippet_id = $1
        "#,
        snippet.id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    Ok(SnippetResponse {
        id: snippet.id,
        title: snippet.title,
        description: snippet.description,
        language: snippet.language,
        is_public: snippet.is_public,
        user: UserResponse::from(user),
        parent_id: snippet.parent_id,
        likes_count: snippet.likes_count,
        forks_count: snippet.forks_count,
        views_count: snippet.views_count,
        files,
        tags,
        created_at: snippet.created_at,
        updated_at: snippet.updated_at,
    })
}

#[derive(Debug, serde::Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/users")
            .route("/{user_id}", web::get().to(get_user))
            .route("/{user_id}/snippets", web::get().to(get_user_snippets))
            .route("/{user_id}/forks", web::get().to(get_user_forks))
    );
}
