use crate::models::{
    User, UserResponse, Claims, Snippet, SnippetResponse,
    CreateSnippet, UpdateSnippet, SnippetFile, CreateFile,
    Version, PaginatedResponse
};
use crate::handlers::users::PaginationQuery;
use actix_web::{web, HttpResponse, Responder, Error};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

pub async fn create_snippet(
    pool: web::Data<PgPool>,
    claims: Claims,
    form: web::Json<CreateSnippet>,
) -> Result<impl Responder, Error> {
    let snippet_id = Uuid::new_v4();
    let version_id = Uuid::new_v4();

    let mut tx = pool.begin().await.map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let snippet = sqlx::query_as!(
        Snippet,
        r#"
        INSERT INTO snippets (id, title, description, language, is_public, user_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, title, description, language, is_public, user_id, parent_id,
                  likes_count, forks_count, views_count, created_at, updated_at
        "#,
        snippet_id,
        form.title,
        form.description,
        form.language,
        form.is_public,
        claims.sub
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to create snippet")
    })?;

    let version = sqlx::query_as!(
        Version,
        r#"
        INSERT INTO versions (id, snippet_id, version_number, commit_message, user_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, snippet_id, version_number, commit_message, user_id, created_at
        "#,
        version_id,
        snippet_id,
        1,
        Some("Initial version".to_string()),
        claims.sub
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to create version")
    })?;

    for file in &form.files {
        sqlx::query!(
            r#"
            INSERT INTO snippet_files (snippet_id, version_id, filename, content, language)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            snippet_id,
            version_id,
            file.filename,
            file.content,
            file.language
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to create file")
        })?;
    }

    for tag_name in &form.tags {
        let tag = sqlx::query!(
            r#"
            INSERT INTO tags (name) VALUES ($1)
            ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
            RETURNING id
            "#,
            tag_name
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to create tag")
        })?;

        sqlx::query!(
            r#"
            INSERT INTO snippet_tags (snippet_id, tag_id) VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
            snippet_id,
            tag.id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to associate tag")
        })?;
    }

    tx.commit().await.map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to commit transaction")
    })?;

    let response = build_snippet_response(&pool, snippet).await?;

    Ok(HttpResponse::Created().json(response))
}

pub async fn update_snippet(
    pool: web::Data<PgPool>,
    claims: Claims,
    snippet_id: web::Path<Uuid>,
    form: web::Json<UpdateSnippet>,
) -> Result<impl Responder, Error> {
    let snippet_id = snippet_id.into_inner();

    let existing = sqlx::query_as!(
        Snippet,
        r#"
        SELECT id, title, description, language, is_public, user_id, parent_id,
               likes_count, forks_count, views_count, created_at, updated_at
        FROM snippets WHERE id = $1
        "#,
        snippet_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let snippet = match existing {
        Some(s) if s.user_id == claims.sub => s,
        Some(_) => return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "forbidden",
            "message": "You don't have permission to update this snippet"
        }))),
        None => return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found",
            "message": "Snippet not found"
        }))),
    };

    let mut tx = pool.begin().await.map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let current_version: i32 = sqlx::query_scalar!(
        r#"SELECT MAX(version_number) FROM versions WHERE snippet_id = $1"#,
        snippet_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?
    .unwrap_or(0);

    let new_version_number = current_version + 1;
    let new_version_id = Uuid::new_v4();

    let title = form.title.as_ref().unwrap_or(&snippet.title);
    let description = form.description.as_ref().or(snippet.description.as_ref());
    let language = form.language.as_ref().unwrap_or(&snippet.language);
    let is_public = form.is_public.unwrap_or(snippet.is_public);

    let updated_snippet = sqlx::query_as!(
        Snippet,
        r#"
        UPDATE snippets
        SET title = $1, description = $2, language = $3, is_public = $4, updated_at = NOW()
        WHERE id = $5
        RETURNING id, title, description, language, is_public, user_id, parent_id,
                  likes_count, forks_count, views_count, created_at, updated_at
        "#,
        title,
        description,
        language,
        is_public,
        snippet_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to update snippet")
    })?;

    let commit_message = form.commit_message.clone()
        .unwrap_or_else(|| format!("Updated to version {}", new_version_number));

    let _version = sqlx::query_as!(
        Version,
        r#"
        INSERT INTO versions (id, snippet_id, version_number, commit_message, user_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, snippet_id, version_number, commit_message, user_id, created_at
        "#,
        new_version_id,
        snippet_id,
        new_version_number,
        Some(commit_message),
        claims.sub
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to create version")
    })?;

    if let Some(files) = &form.files {
        for file in files {
            sqlx::query!(
                r#"
                INSERT INTO snippet_files (snippet_id, version_id, filename, content, language)
                VALUES ($1, $2, $3, $4, $5)
                "#,
                snippet_id,
                new_version_id,
                file.filename,
                file.content,
                file.language
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                eprintln!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to create file")
            })?;
        }
    } else {
        let old_files = sqlx::query_as!(
            SnippetFile,
            r#"
            SELECT id, snippet_id, version_id, filename, content, language, created_at
            FROM snippet_files WHERE snippet_id = $1 AND version_id = (
                SELECT id FROM versions WHERE snippet_id = $1 ORDER BY version_number DESC LIMIT 1
            )
            "#,
            snippet_id
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

        for file in old_files {
            sqlx::query!(
                r#"
                INSERT INTO snippet_files (snippet_id, version_id, filename, content, language)
                VALUES ($1, $2, $3, $4, $5)
                "#,
                snippet_id,
                new_version_id,
                file.filename,
                file.content,
                file.language
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                eprintln!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to copy file")
            })?;
        }
    }

    if let Some(tags) = &form.tags {
        sqlx::query!(
            r#"DELETE FROM snippet_tags WHERE snippet_id = $1"#,
            snippet_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to clear tags")
        })?;

        for tag_name in tags {
            let tag = sqlx::query!(
                r#"
                INSERT INTO tags (name) VALUES ($1)
                ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
                RETURNING id
                "#,
                tag_name
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| {
                eprintln!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to create tag")
            })?;

            sqlx::query!(
                r#"
                INSERT INTO snippet_tags (snippet_id, tag_id) VALUES ($1, $2)
                "#,
                snippet_id,
                tag.id
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                eprintln!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to associate tag")
            })?;
        }
    }

    tx.commit().await.map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to commit transaction")
    })?;

    let response = build_snippet_response(&pool, updated_snippet).await?;

    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_snippet(
    pool: web::Data<PgPool>,
    snippet_id: web::Path<Uuid>,
    claims: Option<Claims>,
) -> Result<impl Responder, Error> {
    let snippet_id = snippet_id.into_inner();

    let snippet = sqlx::query_as!(
        Snippet,
        r#"
        SELECT id, title, description, language, is_public, user_id, parent_id,
               likes_count, forks_count, views_count, created_at, updated_at
        FROM snippets WHERE id = $1
        "#,
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
        Some(s) if claims.as_ref().map(|c| c.sub == s.user_id).unwrap_or(false) => s,
        Some(_) => return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "forbidden",
            "message": "This snippet is private"
        }))),
        None => return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found",
            "message": "Snippet not found"
        }))),
    };

    sqlx::query!(
        r#"UPDATE snippets SET views_count = views_count + 1 WHERE id = $1"#,
        snippet_id
    )
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let response = build_snippet_response(&pool, snippet).await?;

    Ok(HttpResponse::Ok().json(response))
}

pub async fn delete_snippet(
    pool: web::Data<PgPool>,
    claims: Claims,
    snippet_id: web::Path<Uuid>,
) -> Result<impl Responder, Error> {
    let snippet_id = snippet_id.into_inner();

    let existing = sqlx::query!(
        r#"SELECT user_id FROM snippets WHERE id = $1"#,
        snippet_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    match existing {
        Some(s) if s.user_id == claims.sub => {
            sqlx::query!(r#"DELETE FROM snippets WHERE id = $1"#, snippet_id)
                .execute(pool.get_ref())
                .await
                .map_err(|e| {
                    eprintln!("Database error: {}", e);
                    actix_web::error::ErrorInternalServerError("Failed to delete snippet")
                })?;
            Ok(HttpResponse::NoContent().finish())
        }
        Some(_) => Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "forbidden",
            "message": "You don't have permission to delete this snippet"
        }))),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found",
            "message": "Snippet not found"
        }))),
    }
}

pub async fn fork_snippet(
    pool: web::Data<PgPool>,
    claims: Claims,
    snippet_id: web::Path<Uuid>,
) -> Result<impl Responder, Error> {
    let original_id = snippet_id.into_inner();

    let original = sqlx::query_as!(
        Snippet,
        r#"
        SELECT id, title, description, language, is_public, user_id, parent_id,
               likes_count, forks_count, views_count, created_at, updated_at
        FROM snippets WHERE id = $1
        "#,
        original_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let original = match original {
        Some(s) if s.is_public => s,
        Some(_) => return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "forbidden",
            "message": "Cannot fork private snippet"
        }))),
        None => return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found",
            "message": "Snippet not found"
        }))),
    };

    if original.user_id == claims.sub {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "cannot_fork_own",
            "message": "You cannot fork your own snippet"
        })));
    }

    let new_snippet_id = Uuid::new_v4();
    let new_version_id = Uuid::new_v4();

    let mut tx = pool.begin().await.map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let forked_title = format!("{} (Fork)", original.title);

    let new_snippet = sqlx::query_as!(
        Snippet,
        r#"
        INSERT INTO snippets (id, title, description, language, is_public, user_id, parent_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, title, description, language, is_public, user_id, parent_id,
                  likes_count, forks_count, views_count, created_at, updated_at
        "#,
        new_snippet_id,
        forked_title,
        original.description,
        original.language,
        true,
        claims.sub,
        original_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to create fork")
    })?;

    sqlx::query!(
        r#"UPDATE snippets SET forks_count = forks_count + 1 WHERE id = $1"#,
        original_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to update forks count")
    })?;

    let version = sqlx::query_as!(
        Version,
        r#"
        INSERT INTO versions (id, snippet_id, version_number, commit_message, user_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, snippet_id, version_number, commit_message, user_id, created_at
        "#,
        new_version_id,
        new_snippet_id,
        1,
        Some(format!("Forked from {}", original_id)),
        claims.sub
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to create version")
    })?;

    let original_files = sqlx::query_as!(
        SnippetFile,
        r#"
        SELECT id, snippet_id, version_id, filename, content, language, created_at
        FROM snippet_files WHERE snippet_id = $1
        AND version_id = (SELECT id FROM versions WHERE snippet_id = $1 ORDER BY version_number DESC LIMIT 1)
        "#,
        original_id
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    for file in original_files {
        sqlx::query!(
            r#"
            INSERT INTO snippet_files (snippet_id, version_id, filename, content, language)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            new_snippet_id,
            new_version_id,
            file.filename,
            file.content,
            file.language
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to copy file")
        })?;
    }

    let original_tags: Vec<String> = sqlx::query_scalar!(
        r#"
        SELECT t.name FROM tags t
        JOIN snippet_tags st ON t.id = st.tag_id
        WHERE st.snippet_id = $1
        "#,
        original_id
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    for tag_name in original_tags {
        let tag = sqlx::query!(
            r#"
            INSERT INTO tags (name) VALUES ($1)
            ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
            RETURNING id
            "#,
            tag_name
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to create tag")
        })?;

        sqlx::query!(
            r#"
            INSERT INTO snippet_tags (snippet_id, tag_id) VALUES ($1, $2)
            "#,
            new_snippet_id,
            tag.id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to associate tag")
        })?;
    }

    tx.commit().await.map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to commit transaction")
    })?;

    let response = build_snippet_response(&pool, new_snippet).await?;

    Ok(HttpResponse::Created().json(response))
}

pub(crate) async fn build_snippet_response(
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
        SnippetFile,
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

pub async fn get_hot_snippets(
    pool: web::Data<PgPool>,
    query: web::Query<PaginationQuery>,
) -> Result<impl Responder, Error> {
    let page = query.page.unwrap_or(1) as i64;
    let per_page = query.per_page.unwrap_or(20).min(100) as i64;
    let offset = (page - 1) * per_page;

    let snippets = sqlx::query_as!(
        Snippet,
        r#"
        SELECT id, title, description, language, is_public, user_id, parent_id,
               likes_count, forks_count, views_count, created_at, updated_at
        FROM snippets
        WHERE is_public = true
        ORDER BY likes_count DESC, created_at DESC
        LIMIT $1 OFFSET $2
        "#,
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
        r#"SELECT COUNT(*) FROM snippets WHERE is_public = true"#
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

    let total_pages = (total + per_page - 1) / per_page;

    Ok(HttpResponse::Ok().json(PaginatedResponse {
        data: snippet_responses,
        total,
        page,
        per_page,
        total_pages,
    }))
}

pub async fn get_latest_snippets(
    pool: web::Data<PgPool>,
    query: web::Query<PaginationQuery>,
) -> Result<impl Responder, Error> {
    let page = query.page.unwrap_or(1) as i64;
    let per_page = query.per_page.unwrap_or(20).min(100) as i64;
    let offset = (page - 1) * per_page;

    let snippets = sqlx::query_as!(
        Snippet,
        r#"
        SELECT id, title, description, language, is_public, user_id, parent_id,
               likes_count, forks_count, views_count, created_at, updated_at
        FROM snippets
        WHERE is_public = true
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
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
        r#"SELECT COUNT(*) FROM snippets WHERE is_public = true"#
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

    let total_pages = (total + per_page - 1) / per_page;

    Ok(HttpResponse::Ok().json(PaginatedResponse {
        data: snippet_responses,
        total,
        page,
        per_page,
        total_pages,
    }))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/snippets")
            .route("/hot", web::get().to(get_hot_snippets))
            .route("/latest", web::get().to(get_latest_snippets))
            .route("", web::post().to(create_snippet).wrap(crate::middleware::auth::Auth))
            .route("/{snippet_id}", web::get().to(get_snippet).wrap(crate::middleware::auth::OptionalAuth))
            .route("/{snippet_id}", web::put().to(update_snippet).wrap(crate::middleware::auth::Auth))
            .route("/{snippet_id}", web::delete().to(delete_snippet).wrap(crate::middleware::auth::Auth))
            .route("/{snippet_id}/fork", web::post().to(fork_snippet).wrap(crate::middleware::auth::Auth))
    );
}
