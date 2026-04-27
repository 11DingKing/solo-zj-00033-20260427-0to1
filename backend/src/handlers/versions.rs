use crate::models::{Version, SnippetFile, Claims, Snippet};
use actix_web::{web, HttpResponse, Responder, Error};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_versions(
    pool: web::Data<PgPool>,
    snippet_id: web::Path<Uuid>,
) -> Result<impl Responder, Error> {
    let snippet_id = snippet_id.into_inner();

    let snippet = sqlx::query!(
        r#"SELECT id, is_public, user_id FROM snippets WHERE id = $1"#,
        snippet_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    match snippet {
        Some(_) => {
            let versions = sqlx::query_as!(
                Version,
                r#"
                SELECT id, snippet_id, version_number, commit_message, user_id, created_at
                FROM versions WHERE snippet_id = $1
                ORDER BY version_number DESC
                "#,
                snippet_id
            )
            .fetch_all(pool.get_ref())
            .await
            .map_err(|e| {
                eprintln!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;

            Ok(HttpResponse::Ok().json(versions))
        }
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found",
            "message": "Snippet not found"
        }))),
    }
}

pub async fn get_version(
    pool: web::Data<PgPool>,
    path: web::Path<(Uuid, i32)>,
) -> Result<impl Responder, Error> {
    let (snippet_id, version_number) = path.into_inner();

    let version = sqlx::query_as!(
        Version,
        r#"
        SELECT id, snippet_id, version_number, commit_message, user_id, created_at
        FROM versions WHERE snippet_id = $1 AND version_number = $2
        "#,
        snippet_id,
        version_number
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let version = match version {
        Some(v) => v,
        None => return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found",
            "message": "Version not found"
        }))),
    };

    let files = sqlx::query_as!(
        SnippetFile,
        r#"
        SELECT id, snippet_id, version_id, filename, content, language, created_at
        FROM snippet_files WHERE snippet_id = $1 AND version_id = $2
        "#,
        snippet_id,
        version.id
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "version": version,
        "files": files
    })))
}

pub async fn get_version_diff(
    pool: web::Data<PgPool>,
    path: web::Path<(Uuid, i32, i32)>,
) -> Result<impl Responder, Error> {
    let (snippet_id, version_a, version_b) = path.into_inner();

    let version_a_data = sqlx::query_as!(
        Version,
        r#"
        SELECT id, snippet_id, version_number, commit_message, user_id, created_at
        FROM versions WHERE snippet_id = $1 AND version_number = $2
        "#,
        snippet_id,
        version_a
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let version_b_data = sqlx::query_as!(
        Version,
        r#"
        SELECT id, snippet_id, version_number, commit_message, user_id, created_at
        FROM versions WHERE snippet_id = $1 AND version_number = $2
        "#,
        snippet_id,
        version_b
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let version_a_data = match version_a_data {
        Some(v) => v,
        None => return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found",
            "message": format!("Version {} not found", version_a)
        }))),
    };

    let version_b_data = match version_b_data {
        Some(v) => v,
        None => return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found",
            "message": format!("Version {} not found", version_b)
        }))),
    };

    let files_a = sqlx::query_as!(
        SnippetFile,
        r#"
        SELECT id, snippet_id, version_id, filename, content, language, created_at
        FROM snippet_files WHERE version_id = $1
        "#,
        version_a_data.id
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let files_b = sqlx::query_as!(
        SnippetFile,
        r#"
        SELECT id, snippet_id, version_id, filename, content, language, created_at
        FROM snippet_files WHERE version_id = $1
        "#,
        version_b_data.id
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "version_a": version_a_data,
        "version_b": version_b_data,
        "files_a": files_a,
        "files_b": files_b
    })))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/snippets")
            .route("/{snippet_id}/versions", web::get().to(get_versions))
            .route("/{snippet_id}/versions/{version_number}", web::get().to(get_version))
            .route("/{snippet_id}/diff/{version_a}/{version_b}", web::get().to(get_version_diff))
    );
}
