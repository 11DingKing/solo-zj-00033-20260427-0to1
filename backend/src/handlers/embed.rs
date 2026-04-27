use crate::models::{Snippet, SnippetFile, User};
use actix_web::{web, HttpResponse, Responder, Error};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn embed_snippet(
    pool: web::Data<PgPool>,
    snippet_id: web::Path<Uuid>,
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
        Some(_) => {
            let html = r#"
                <!DOCTYPE html>
                <html>
                <head><title>Private Snippet</title></head>
                <body style="margin:0;padding:20px;background:#1e1e1e;color:#f0f0f0;font-family:monospace;">
                    <div style="text-align:center;">This snippet is private</div>
                </body></html>
            "#;
            return Ok(HttpResponse::Forbidden().content_type("text/html").body(html));
        }
        None => {
            let html = r#"
                <!DOCTYPE html>
                <html>
                <head><title>Snippet Not Found</title></head>
                <body style="margin:0;padding:20px;background:#1e1e1e;color:#f0f0f0;font-family:monospace;">
                    <div style="text-align:center;">Snippet not found</div>
                </body></html>
            "#;
            return Ok(HttpResponse::NotFound().content_type("text/html").body(html));
        }
    };

    let user = sqlx::query_as!(
        User,
        r#"SELECT id, username, email, password_hash, display_name, avatar_url, created_at, updated_at FROM users WHERE id = $1"#,
        snippet.user_id
    )
    .fetch_optional(pool.get_ref())
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
        snippet_id
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let author_name = user.map(|u| u.display_name.unwrap_or(u.username)).unwrap_or_else(|| "Unknown".to_string());

    let files_html: String = files.iter().map(|file| {
        let language = file.language.as_deref().unwrap_or("text");
        let content = html_escape(&file.content);
        let filename = html_escape(&file.filename);
        
        format!(
            r#"
            <div class="file">
                <div class="file-header">
                    <span class="filename">{filename}</span>
                    <span class="language">{language}</span>
                </div>
                <pre class="code"><code class="language-{language}">{content}</code></pre>
            </div>
            "#
        )
    }).collect();

    let html = format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>{title} by {author}</title>
            <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/themes/prism-tomorrow.min.css">
            <style>
                * {{ margin: 0; padding: 0; box-sizing: border-box; }}
                body {{
                    background: #1e1e1e;
                    color: #d4d4d4;
                    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                    font-size: 14px;
                }}
                .container {{
                    max-width: 100%;
                    margin: 0;
                    padding: 16px;
                }}
                .header {{
                    margin-bottom: 16px;
                    padding-bottom: 12px;
                    border-bottom: 1px solid #333;
                }}
                .title {{
                    font-size: 18px;
                    font-weight: 600;
                    color: #569cd6;
                    margin-bottom: 8px;
                }}
                .meta {{
                    font-size: 12px;
                    color: #858585;
                }}
                .author {{ color: #4ec9b0; }}
                .language {{
                    background: #333;
                    padding: 2px 8px;
                    border-radius: 4px;
                    font-size: 11px;
                    color: #ce9178;
                }}
                .file {{
                    margin-bottom: 16px;
                    border: 1px solid #333;
                    border-radius: 6px;
                    overflow: hidden;
                }}
                .file-header {{
                    background: #2d2d2d;
                    padding: 8px 12px;
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    border-bottom: 1px solid #333;
                }}
                .filename {{
                    font-family: 'Courier New', monospace;
                    font-size: 13px;
                    color: #dcdcaa;
                }}
                pre {{
                    margin: 0 !important;
                    padding: 12px !important;
                    background: #1e1e1e !important;
                    overflow-x: auto;
                    border-radius: 0 !important;
                }}
                code {{
                    font-family: 'Courier New', monospace;
                    font-size: 13px;
                    line-height: 1.5;
                }}
            </style>
        </head>
        <body>
            <div class="container">
                <div class="header">
                    <div class="title">{title}</div>
                    <div class="meta">
                        by <span class="author">{author}</span> · 
                        <span class="language">{language}</span>
                    </div>
                </div>
                {files_html}
            </div>
            <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-core.min.js"></script>
            <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-markup.min.js"></script>
            <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-css.min.js"></script>
            <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-javascript.min.js"></script>
            <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-typescript.min.js"></script>
            <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-python.min.js"></script>
            <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-rust.min.js"></script>
            <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-go.min.js"></script>
            <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-java.min.js"></script>
            <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-csharp.min.js"></script>
            <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-php.min.js"></script>
            <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-ruby.min.js"></script>
            <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-swift.min.js"></script>
            <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-sql.min.js"></script>
            <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-json.min.js"></script>
            <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-yaml.min.js"></script>
            <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-bash.min.js"></script>
        </body>
        </html>
        "#,
        title = html_escape(&snippet.title),
        author = html_escape(&author_name),
        language = html_escape(&snippet.language),
        files_html = files_html
    );

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/embed")
            .route("/{snippet_id}", web::get().to(embed_snippet))
    );
}
