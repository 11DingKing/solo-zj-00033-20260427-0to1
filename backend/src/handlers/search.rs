use crate::models::{Snippet, SnippetResponse, User, UserResponse, SnippetFile, PaginatedResponse};
use crate::handlers::snippets::build_snippet_response;
use actix_web::{web, HttpResponse, Responder, Error};
use sqlx::PgPool;

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
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let search_term = query.q.as_deref().unwrap_or("");
    let language_filter = query.language.as_deref();
    let tags_filter: Vec<&str> = query.tags.as_deref()
        .map(|t| t.split(',').collect())
        .unwrap_or_default();

    let mut conditions = Vec::new();
    let mut params: Vec<&(dyn sqlx::Encode<'_, sqlx::Postgres> + Sync)> = Vec::new();
    let mut param_idx = 1;

    conditions.push("s.is_public = true".to_string());

    if !search_term.is_empty() {
        conditions.push(format!(
            "(s.search_vector @@ plainto_tsquery('english', ${}) OR s.title ILIKE '%' || ${} || '%' OR s.description ILIKE '%' || ${} || '%')",
            param_idx, param_idx, param_idx
        ));
        params.push(&search_term);
        param_idx += 1;
    }

    if let Some(lang) = language_filter {
        conditions.push(format!("s.language = ${}", param_idx));
        params.push(&lang);
        param_idx += 1;
    }

    let where_clause = conditions.join(" AND ");

    let tags_join = if !tags_filter.is_empty() {
        format!(
            "JOIN snippet_tags st ON s.id = st.snippet_id JOIN tags t ON st.tag_id = t.id WHERE {} AND t.name = ANY(${})",
            where_clause, param_idx
        )
    } else {
        format!("WHERE {}", where_clause)
    };

    if !tags_filter.is_empty() {
        params.push(&tags_filter);
        param_idx += 1;
    }

    let snippets = sqlx::query_as!(
        Snippet,
        &format!(
            r#"
            SELECT DISTINCT s.id, s.title, s.description, s.language, s.is_public, s.user_id, s.parent_id,
                   s.likes_count, s.forks_count, s.views_count, s.created_at, s.updated_at
            FROM snippets s
            {}
            ORDER BY s.likes_count DESC, s.created_at DESC
            LIMIT ${} OFFSET ${}
            "#,
            tags_join,
            param_idx,
            param_idx + 1
        ),
        search_term,
        per_page,
        offset
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let total_query = format!(
        r#"
        SELECT COUNT(DISTINCT s.id) FROM snippets s
        {}
        "#,
        tags_join
    );

    let total: i64 = sqlx::query_scalar(&total_query)
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
