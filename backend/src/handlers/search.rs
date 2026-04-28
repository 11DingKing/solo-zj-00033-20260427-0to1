use crate::models::{Snippet, SnippetResponse, PaginatedResponse, SnippetFile, User, UserResponse};
use crate::handlers::snippets::build_snippet_response;
use actix_web::{web, HttpResponse, Responder, Error};
use sqlx::{PgPool, Postgres, Row};

#[derive(Debug, serde::Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub language: Option<String>,
    pub tags: Option<String>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

pub async fn search_snippets(
    pool: web::Data<PgPool>,
    query: web::Query<SearchQuery>,
) -> Result<impl Responder, Error> {
    let page = query.page.unwrap_or(1) as i64;
    let per_page = query.per_page.unwrap_or(20).min(100) as i64;
    let offset = (page - 1) * per_page;

    let search_term = query.q.clone().unwrap_or_default();
    let language_filter = query.language.clone();
    let tags_filter: Vec<String> = query.tags.as_deref()
        .map(|t| t.split(',').map(|s| s.to_string()).collect())
        .unwrap_or_default();

    let snippets: Vec<Snippet>;
    let total: i64;

    if tags_filter.is_empty() {
        let sql = r#"
            SELECT s.id, s.title, s.description, s.language, s.is_public, s.user_id, s.parent_id,
                   s.likes_count, s.forks_count, s.views_count, s.created_at, s.updated_at
            FROM snippets s
            WHERE s.is_public = true
              AND ($1 = '' OR s.search_vector @@ plainto_tsquery('english', $1) OR s.title ILIKE '%' || $1 || '%' OR s.description ILIKE '%' || $1 || '%')
              AND ($2 IS NULL OR s.language = $2)
            ORDER BY s.likes_count DESC, s.created_at DESC
            LIMIT $3 OFFSET $4
        "#;

        let total_sql = r#"
            SELECT COUNT(DISTINCT s.id) FROM snippets s
            WHERE s.is_public = true
              AND ($1 = '' OR s.search_vector @@ plainto_tsquery('english', $1) OR s.title ILIKE '%' || $1 || '%' OR s.description ILIKE '%' || $1 || '%')
              AND ($2 IS NULL OR s.language = $2)
        "#;

        snippets = sqlx::query_as::<_, Snippet>(sql)
            .bind(&search_term)
            .bind(&language_filter)
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool.get_ref())
            .await
            .map_err(|e| {
                eprintln!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;

        total = sqlx::query_scalar::<_, i64>(total_sql)
            .bind(&search_term)
            .bind(&language_filter)
            .fetch_one(pool.get_ref())
            .await
            .map_err(|e| {
                eprintln!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
    } else {
        let sql = r#"
            SELECT DISTINCT s.id, s.title, s.description, s.language, s.is_public, s.user_id, s.parent_id,
                   s.likes_count, s.forks_count, s.views_count, s.created_at, s.updated_at
            FROM snippets s
            JOIN snippet_tags st ON s.id = st.snippet_id
            JOIN tags t ON st.tag_id = t.id
            WHERE s.is_public = true
              AND ($1 = '' OR s.search_vector @@ plainto_tsquery('english', $1) OR s.title ILIKE '%' || $1 || '%' OR s.description ILIKE '%' || $1 || '%')
              AND ($2 IS NULL OR s.language = $2)
              AND t.name = ANY($3)
            ORDER BY s.likes_count DESC, s.created_at DESC
            LIMIT $4 OFFSET $5
        "#;

        let total_sql = r#"
            SELECT COUNT(DISTINCT s.id) FROM snippets s
            JOIN snippet_tags st ON s.id = st.snippet_id
            JOIN tags t ON st.tag_id = t.id
            WHERE s.is_public = true
              AND ($1 = '' OR s.search_vector @@ plainto_tsquery('english', $1) OR s.title ILIKE '%' || $1 || '%' OR s.description ILIKE '%' || $1 || '%')
              AND ($2 IS NULL OR s.language = $2)
              AND t.name = ANY($3)
        "#;

        snippets = sqlx::query_as::<_, Snippet>(sql)
            .bind(&search_term)
            .bind(&language_filter)
            .bind(&tags_filter)
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool.get_ref())
            .await
            .map_err(|e| {
                eprintln!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;

        total = sqlx::query_scalar::<_, i64>(total_sql)
            .bind(&search_term)
            .bind(&language_filter)
            .bind(&tags_filter)
            .fetch_one(pool.get_ref())
            .await
            .map_err(|e| {
                eprintln!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
    }

    let mut snippet_responses = Vec::new();
    for snippet in snippets {
        let response = build_snippet_response(&pool, snippet).await?;
        snippet_responses.push(response);
    }

    let total_pages = (total + per_page - 1) / per_page;

    Ok(HttpResponse::Ok().json(PaginatedResponse {
        data: snippet_responses,
        total,
        page,
        per_page,
        total_pages,
    }))
}

pub async fn get_languages(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, Error> {
    let languages: Vec<String> = sqlx::query_scalar!(
        r#"SELECT DISTINCT language FROM snippets WHERE is_public = true ORDER BY language"#
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    Ok(HttpResponse::Ok().json(languages))
}

pub async fn get_popular_tags(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, Error> {
    let tags = sqlx::query!(
        r#"
        SELECT t.name, COUNT(st.snippet_id) as count
        FROM tags t
        JOIN snippet_tags st ON t.id = st.tag_id
        JOIN snippets s ON st.snippet_id = s.id
        WHERE s.is_public = true
        GROUP BY t.name
        ORDER BY count DESC
        LIMIT 50
        "#
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let result: Vec<serde_json::Value> = tags.into_iter().map(|t| {
        serde_json::json!({
            "name": t.name,
            "count": t.count
        })
    }).collect();

    Ok(HttpResponse::Ok().json(result))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/search")
            .route("", web::get().to(search_snippets))
            .route("/languages", web::get().to(get_languages))
            .route("/tags", web::get().to(get_popular_tags))
    );
}
