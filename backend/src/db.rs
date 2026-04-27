use sqlx::PgPool;
use std::fs;
use std::path::Path;

pub async fn create_pool(database_url: &str) -> PgPool {
    PgPool::connect(database_url)
        .await
        .expect("Failed to create database pool")
}

pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    let migrations_path = Path::new("./migrations");
    
    if !migrations_path.exists() {
        println!("Migrations directory not found, skipping migrations");
        return Ok(());
    }

    let mut entries: Vec<_> = fs::read_dir(migrations_path)?
        .filter_map(|e| e.ok())
        .collect();
    
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if path.extension().map(|e| e == "sql").unwrap_or(false) {
            let sql = fs::read_to_string(&path)?;
            
            let statements: Vec<&str> = sql
                .split(';')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            for stmt in statements {
                if !stmt.is_empty() {
                    sqlx::query(stmt)
                        .execute(pool)
                        .await?;
                }
            }
            println!("Executed migration: {:?}", entry.file_name());
        }
    }

    Ok(())
}
