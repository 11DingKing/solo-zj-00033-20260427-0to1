use crate::models::{Claims, Like};
use actix_web::{web, HttpResponse, Responder, Error};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn toggle_like(
    pool: web::Data<PgPool>,
    claims: Claims,
    snippet_id: web::Path<Uuid>,
) -> Result<impl Responder, Error> {
    let snippet_id = snippet_id.into_inner();

    let snippet = sqlx::query!(
        r#"SELECT id, is_public, likes_count FROM snippets WHERE id = $1"#,
        snippet_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let snippet = match snippet {
        Some(s) if s.is_public => s,
        Some(_) => return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "forbidden",
            "message": "Cannot like private snippet"
        }))),
        None => return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found",
            "message": "Snippet not found"
        }))),
    };

    let existing_like = sqlx::query_as!(
        Like,
        r#"
        SELECT id, user_id, snippet_id, created_at
        FROM likes WHERE user_id = $1 AND snippet_id = $2
        "#,
        claims.sub,
        snippet_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let is_liked: bool;

    if let Some(_) = existing_like {
        sqlx::query!(
            r#"DELETE FROM likes WHERE user_id = $1 AND snippet_id = $2"#,
            claims.sub,
            snippet_id
        )
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to remove like")
        })?;

        sqlx::query!(
            r#"UPDATE snippets SET likes_count = likes_count - 1 WHERE id = $1"#,
            snippet_id
        )
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to update likes count")
        })?;

        is_liked = false;
    } else {
        let like_id = Uuid::new_v4();
        
        sqlx::query!(
            r#"INSERT INTO likes (id, user_id, snippet_id) VALUES ($1, $2, $3)"#,
            like_id,
            claims.sub,
            snippet_id
        )
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to add like")
        })?;

        sqlx::query!(
            r#"UPDATE snippets SET likes_count = likes_count + 1 WHERE id = $1"#,
            snippet_id
        )
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to update likes count")
        })?;

        is_liked = true;
    }

    let updated_snippet = sqlx::query!(
        r#"SELECT likes_count FROM snippets WHERE id = $1"#,
        snippet_id
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "liked": is_liked,
        "likes_count": updated_snippet.likes_count
    })))
}

pub async fn check_like(
    pool: web::Data<PgPool>,
    claims: Claims,
    snippet_id: web::Path<Uuid>,
) -> Result<impl Responder, Error> {
    let snippet_id = snippet_id.into_inner();

    let existing_like = sqlx::query_as!(
        Like,
        r#"
        SELECT id, user_id, snippet_id, created_at
        FROM likes WHERE user_id = $1 AND snippet_id = $2
        "#,
        claims.sub,
        snippet_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "liked": existing_like.is_some()
    })))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/snippets")
            .route("/{snippet_id}/like", web::post().to(toggle_like).wrap(crate::middleware::auth::Auth))
            .route("/{snippet_id}/like", web::get().to(check_like).wrap(crate::middleware::auth::Auth))
    );
}
