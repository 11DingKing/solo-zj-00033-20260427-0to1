use crate::models::{Comment, CreateComment, Claims, User, UserResponse};
use actix_web::{web, HttpResponse, Responder, Error};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_comments(
    pool: web::Data<PgPool>,
    snippet_id: web::Path<Uuid>,
) -> Result<impl Responder, Error> {
    let snippet_id = snippet_id.into_inner();

    let comments = sqlx::query_as!(
        Comment,
        r#"
        SELECT id, snippet_id, user_id, content, parent_id, created_at, updated_at
        FROM comments WHERE snippet_id = $1
        ORDER BY created_at ASC
        "#,
        snippet_id
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let mut enriched_comments = Vec::new();
    for comment in comments {
        let user = sqlx::query_as!(
            User,
            r#"SELECT id, username, email, password_hash, display_name, avatar_url, created_at, updated_at FROM users WHERE id = $1"#,
            comment.user_id
        )
        .fetch_optional(pool.get_ref())
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

        enriched_comments.push(serde_json::json!({
            "id": comment.id,
            "snippet_id": comment.snippet_id,
            "user": user.map(UserResponse::from),
            "content": comment.content,
            "parent_id": comment.parent_id,
            "created_at": comment.created_at,
            "updated_at": comment.updated_at
        }));
    }

    Ok(HttpResponse::Ok().json(enriched_comments))
}

pub async fn create_comment(
    pool: web::Data<PgPool>,
    claims: Claims,
    snippet_id: web::Path<Uuid>,
    form: web::Json<CreateComment>,
) -> Result<impl Responder, Error> {
    let snippet_id = snippet_id.into_inner();

    let snippet = sqlx::query!(
        r#"SELECT id, is_public FROM snippets WHERE id = $1"#,
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
            "message": "Cannot comment on private snippet"
        }))),
        None => return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found",
            "message": "Snippet not found"
        }))),
    };

    let comment_id = Uuid::new_v4();

    let comment = sqlx::query_as!(
        Comment,
        r#"
        INSERT INTO comments (id, snippet_id, user_id, content, parent_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, snippet_id, user_id, content, parent_id, created_at, updated_at
        "#,
        comment_id,
        snippet_id,
        claims.sub,
        form.content,
        form.parent_id
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to create comment")
    })?;

    let user = sqlx::query_as!(
        User,
        r#"SELECT id, username, email, password_hash, display_name, avatar_url, created_at, updated_at FROM users WHERE id = $1"#,
        claims.sub
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let response = serde_json::json!({
        "id": comment.id,
        "snippet_id": comment.snippet_id,
        "user": UserResponse::from(user),
        "content": comment.content,
        "parent_id": comment.parent_id,
        "created_at": comment.created_at,
        "updated_at": comment.updated_at
    });

    Ok(HttpResponse::Created().json(response))
}

pub async fn delete_comment(
    pool: web::Data<PgPool>,
    claims: Claims,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<impl Responder, Error> {
    let (snippet_id, comment_id) = path.into_inner();

    let existing = sqlx::query!(
        r#"SELECT user_id FROM comments WHERE id = $1 AND snippet_id = $2"#,
        comment_id,
        snippet_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    match existing {
        Some(c) if c.user_id == claims.sub => {
            sqlx::query!(r#"DELETE FROM comments WHERE id = $1"#, comment_id)
                .execute(pool.get_ref())
                .await
                .map_err(|e| {
                    eprintln!("Database error: {}", e);
                    actix_web::error::ErrorInternalServerError("Failed to delete comment")
                })?;
            Ok(HttpResponse::NoContent().finish())
        }
        Some(_) => Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "forbidden",
            "message": "You don't have permission to delete this comment"
        }))),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found",
            "message": "Comment not found"
        }))),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/snippets")
            .route("/{snippet_id}/comments", web::get().to(get_comments))
            .route("/{snippet_id}/comments", web::post().to(create_comment).wrap(crate::middleware::auth::Auth))
            .route("/{snippet_id}/comments/{comment_id}", web::delete().to(delete_comment).wrap(crate::middleware::auth::Auth))
    );
}
