// src/db/database.rs
use sqlx::{PgPool, postgres::PgPoolOptions};
use crate::db::schemas::{ClipboardEntry, NewClipboardEntry, UpdateClipboardEntry};
use crate::config::{get_database_url};
use serde_json;
use std::time::Duration;

pub async fn create_db_pool() -> Result<PgPool, Box<dyn std::error::Error>> {   
    let database_url = get_database_url();
    println!("Connecting to database...");

    let connect_result = tokio::time::timeout(
        Duration::from_secs(40), 
        PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(30))
            .connect(&database_url),
    )
    .await;
    
    let pool = match connect_result {
        // Connected successfully
        Ok(Ok(pool)) => {
            println!("✅ Postgres connected successfully!");
            pool
        }
        // SQLx error inside timeout
        Ok(Err(e)) => {
            eprintln!("❌ Postgres connect failed: {e}");
            return Err(format!("Postgres connect failed: {e}").into());
        }

        Err(_) => {
            eprintln!("⚠️ Postgres connect TIMED OUT (3s). Running in local-only mode.");
            return Err("Postgres connect timed out".into());
        }
    };
    
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await?;    
    println!("Database connected successfully!");    
    // Create tables if they don't exist
    // create_tables(&pool).await?;    
    Ok(pool)
}

pub async fn create_tables(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Creating database tables if they don't exist...");

    println!("📝 Creating purge_cadence enum type if not exists...");
    sqlx::query(
        r#"
        DO $$ 
        BEGIN
            CREATE TYPE purge_cadence AS ENUM (
                'never',
                'every_24_hours', 
                'every_3_days',
                'every_week',
                'every_month'
            );
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#
    )
    .execute(pool)
    .await?;

    println!("📝 Creating Users table if not exists...");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id BIGSERIAL PRIMARY KEY,
            organization_id VARCHAR(255) NOT NULL,
            firebase_uid TEXT UNIQUE NOT NULL,
            email TEXT NOT NULL,
            display_name TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            purge_cadence purge_cadence NOT NULL DEFAULT 'every_24_hours',
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            last_login_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            retain_tags BOOLEAN NOT NULL DEFAULT FALSE
        )
        "#
    )
    .execute(pool)
    .await?;

    println!("📝 Creating clipboard table if not exists...");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS clipboard_entries (
            id BIGSERIAL PRIMARY KEY,
            organization_id VARCHAR(255) NOT NULL,
            content TEXT NOT NULL,
            content_type VARCHAR(50) NOT NULL DEFAULT 'text',
            content_hash VARCHAR(64) UNIQUE NOT NULL,
            source_app VARCHAR(255) NOT NULL,
            source_window VARCHAR(255) NOT NULL,
            timestamp TIMESTAMPTZ NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            tags TEXT,
            is_pinned BOOLEAN NOT NULL DEFAULT FALSE
        )
        "#
    )
    .execute(pool)
    .await?;

    println!("📝 Creating Tags table if not exists...");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tags (
            id BIGSERIAL PRIMARY KEY,
            organization_id VARCHAR(255) NOT NULL,
            name VARCHAR(100) NOT NULL,
            color VARCHAR(7) NOT NULL DEFAULT '#6B7280',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // === Indexes ===
    println!("📝 Creating indexes if not exist...");
    
    // Users indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_firebase_uid ON users(firebase_uid)")
        .execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)")
        .execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_organization_id ON users(organization_id)")
        .execute(pool).await?;

    // Clipboard indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_clipboard_content_hash ON clipboard_entries(content_hash)")
        .execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_clipboard_created_at ON clipboard_entries(created_at DESC)")
        .execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_clipboard_source_app ON clipboard_entries(source_app)")
        .execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_clipboard_organization_id ON clipboard_entries(organization_id)")
        .execute(pool).await?;

    // Tags indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_tags_organization_id ON tags(organization_id)")
        .execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name)")
        .execute(pool).await?;
    sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS idx_tags_organization_name_unique ON tags(organization_id, LOWER(name))")
        .execute(pool).await?;
    
    println!("✅ Database tables ready!");
    Ok(())
}

pub fn tags_to_json(tag_names: &[String]) -> Option<String> {
    if tag_names.is_empty() {
        None
    } else {
        // Use the same serialization logic as the ClipboardEntry model
        match serde_json::to_string(tag_names) {
            Ok(json) => Some(json),
            Err(_) => {
                // Fallback: manual JSON creation (same as in set_tags)
                let tags_json = tag_names
                    .iter()
                    .map(|tag| format!("\"{}\"", tag.replace('\"', "\\\"")))
                    .collect::<Vec<_>>()
                    .join(",");
                Some(format!("[{}]", tags_json))
            }
        }
    }
}

pub fn json_to_tags(tags_json: &Option<String>) -> Vec<String> {
    match tags_json {
        Some(json) if !json.trim().is_empty() => {
            // Use the same parsing logic as the ClipboardEntry model
            let cleaned_json = json
                .trim()
                .replace("\\\"", "\"")
                .replace("\\\\", "\\");
            
            match serde_json::from_str::<Vec<String>>(&cleaned_json) {
                Ok(tags) => tags,
                Err(_) => {
                    if cleaned_json.starts_with('[') && cleaned_json.ends_with(']') {
                        let inner = &cleaned_json[1..cleaned_json.len()-1];
                        inner.split(',')
                            .map(|s| s.trim().trim_matches('"').to_string())
                            .filter(|s| !s.is_empty())
                            .collect()
                    } else {
                        vec![cleaned_json.to_string()]
                    }
                }
            }
        }
        _ => Vec::new(),
    }
}


// Clipboard operations
pub struct ClipboardRepository;

impl ClipboardRepository {
    
 pub async fn save_entry(
        pool: &PgPool, 
        entry: NewClipboardEntry
    ) -> Result<ClipboardEntry, Box<dyn std::error::Error>> {
        // Idempotent upsert by content_hash
        let result = sqlx::query_as::<_, ClipboardEntry>(
            r#"
            INSERT INTO clipboard_entries 
                (content, content_type, content_hash, source_app, source_window, timestamp, tags, organization_id, is_pinned)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (content_hash) DO UPDATE
            SET
                content        = EXCLUDED.content,
                content_type   = EXCLUDED.content_type,
                source_app     = EXCLUDED.source_app,
                source_window  = EXCLUDED.source_window,
                timestamp      = EXCLUDED.timestamp,
                tags           = COALESCE(EXCLUDED.tags, clipboard_entries.tags),
                organization_id = EXCLUDED.organization_id,
                is_pinned    =  EXCLUDED.is_pinned
            RETURNING *
            "#
        )
        .bind(entry.content)
        .bind(entry.content_type)
        .bind(entry.content_hash)
        .bind(entry.source_app)
        .bind(entry.source_window)
        .bind(entry.timestamp)
        .bind(entry.tags)
        .bind(entry.organization_id)
        .bind(entry.is_pinned)
        .fetch_one(pool)
        .await?;
        
        Ok(result)
    }


    pub async fn get_by_organization(
        pool: &PgPool, 
        organization_id: &str,
        limit: Option<i64>
    ) -> Result<Vec<ClipboardEntry>, Box<dyn std::error::Error>> {
        let limit = limit.unwrap_or(100);
        
        let results = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries WHERE organization_id = $1 ORDER BY created_at DESC LIMIT $2"
        )
        .bind(organization_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;
        
        Ok(results)
    }
    
    pub async fn get_by_id(
        pool: &PgPool, 
        id: i64
    ) -> Result<Option<ClipboardEntry>, Box<dyn std::error::Error>> {
        let result = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        
        Ok(result)
    }
    
    pub async fn get_all(
        pool: &PgPool, 
        limit: Option<i64>
    ) -> Result<Vec<ClipboardEntry>, Box<dyn std::error::Error>> {
        let limit = limit.unwrap_or(100);
        
        let results = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries ORDER BY created_at DESC LIMIT $1"
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;
        
        Ok(results)
    }
    
    pub async fn get_recent(
        pool: &PgPool, 
        hours: i32
    ) -> Result<Vec<ClipboardEntry>, Box<dyn std::error::Error>> {
        let results = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries WHERE created_at > NOW() - INTERVAL '1 hour' * $1 ORDER BY created_at DESC"
        )
        .bind(hours)
        .fetch_all(pool)
        .await?;
        
        Ok(results)
    }
    
    pub async fn search_content(
        pool: &PgPool, 
        query: &str
    ) -> Result<Vec<ClipboardEntry>, Box<dyn std::error::Error>> {
        let search_pattern = format!("%{}%", query);
        
        let results = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries WHERE content ILIKE $1 ORDER BY created_at DESC"
        )
        .bind(search_pattern)
        .fetch_all(pool)
        .await?;
        
        Ok(results)
    }
    
    pub async fn update_entry(
        pool: &PgPool, 
        id: i64, 
        update: UpdateClipboardEntry
    ) -> Result<ClipboardEntry, Box<dyn std::error::Error>> {
        let result = sqlx::query_as::<_, ClipboardEntry>(
            r#"
            UPDATE clipboard_entries 
            SET 
                is_pinned = COALESCE($1, is_pinned),
                tags = COALESCE($2, tags)
            WHERE id = $3
            RETURNING *
            "#
        )
        .bind(update.is_pinned)
        .bind(update.tags)
        .bind(id)
        .fetch_one(pool)
        .await?;
        
        Ok(result)
    }
    
    pub async fn delete_entry(
        pool: &PgPool, 
        id: i64
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let result = sqlx::query(
            "DELETE FROM clipboard_entries WHERE id = $1"
        )
        .bind(id)
        .execute(pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }

   pub async fn update_from_local(
        pool: &PgPool,
        server_id: i64,
        local: &ClipboardEntry,
    ) -> Result<ClipboardEntry, sqlx::Error> {
        sqlx::query_as::<_, ClipboardEntry>(
            r#"
            UPDATE clipboard_entries
            SET
                is_pinned = $1,
                tags      = $2,
                timestamp = $3
            WHERE id = $4
            RETURNING *
            "#
        )
        .bind(local.is_pinned)
        .bind(&local.tags)
        .bind(local.timestamp)
        .bind(server_id)
        .fetch_one(pool)
        .await
    }

    pub async fn delete_entry_for_org(
        pool: &PgPool,
        id: i64,
        organization_id: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let result = sqlx::query(
            r#"
            DELETE FROM clipboard_entries
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(organization_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn update_entry_content(
        pool: &PgPool,
        entry_id: i64,
        new_content: &str,
    ) -> Result<ClipboardEntry, Box<dyn std::error::Error>> {
        let content_hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            new_content.hash(&mut hasher);
            format!("{:x}", hasher.finish())
        };
        
        let result = sqlx::query_as::<_, ClipboardEntry>(
            r#"
            UPDATE clipboard_entries 
            SET 
                content = $1,
                content_hash = $2,
                timestamp = $3
            WHERE id = $4
            RETURNING *
            "#
        )
        .bind(new_content)
        .bind(content_hash)
        .bind(chrono::Utc::now())
        .bind(entry_id)
        .fetch_one(pool)
        .await?;
        
        Ok(result)
    }
    
    pub async fn exists_by_hash(
        pool: &PgPool, 
        content_hash: &str
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let result = sqlx::query(
            "SELECT 1 FROM clipboard_entries WHERE content_hash = $1 LIMIT 1"
        )
        .bind(content_hash)
        .fetch_optional(pool)
        .await?;
        
        Ok(result.is_some())
    }




    //Settings commands
    pub async fn delete_entries_older_than(pool: &PgPool, organization_id: &str, days: i32) -> Result<usize, sqlx::Error> {
        sqlx::query(
            "DELETE FROM clipboard_entries WHERE organization_id = $1 AND created_at < NOW() - ($2 || ' days')::INTERVAL"
        )
        .bind(organization_id)
        .bind(days.to_string()) // Convert to string here
        .execute(pool)
        .await
        .map(|result| result.rows_affected() as usize)
    }
    
    pub async fn delete_unpinned_older_than(pool: &PgPool, organization_id: &str, days: i32) -> Result<usize, sqlx::Error> {
        sqlx::query(
            "DELETE FROM clipboard_entries WHERE organization_id = $1 AND is_pinned = false AND created_at < NOW() - ($2 || ' days')::INTERVAL"
        )
        .bind(organization_id)
        .bind(days.to_string()) // Convert to string here
        .execute(pool)
        .await
        .map(|result| result.rows_affected() as usize)
    }

    pub async fn delete_untagged_entries(pool: &PgPool, organization_id: &str) -> Result<usize, sqlx::Error> {
        sqlx::query(
            "DELETE FROM clipboard_entries WHERE organization_id = $1 AND is_pinned = false AND tags IS NULL"
        )
        .bind(organization_id)
        .execute(pool)
        .await
        .map(|result| result.rows_affected() as usize)
    }

    pub async fn delete_unpinned_entries(pool: &PgPool, organization_id: &str) -> Result<usize, sqlx::Error> {
        sqlx::query(
            "DELETE FROM clipboard_entries WHERE organization_id = $1 AND is_pinned = false"
        )
        .bind(organization_id)
        .execute(pool)
        .await
        .map(|result| result.rows_affected() as usize)
    }

    pub async fn assign_tag(
    pool: &PgPool, 
    clipboard_entry_id: i64, 
    tag_name: &str
) -> Result<ClipboardEntry, String> { // Change to String error type
    println!("=== ASSIGN TAG DEBUG ===");
    println!("🟢 Assigning tag '{}' to entry {}", tag_name, clipboard_entry_id);
    
    // First get the current entry
    let current_entry = Self::get_by_id(pool, clipboard_entry_id).await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or("Clipboard entry not found".to_string())?;
    
    println!("📋 Current entry tags: {:?}", current_entry.tags);
    
    // Parse current tags
    let mut current_tags = json_to_tags(&current_entry.tags);
    println!("📋 Parsed current tags: {:?}", current_tags);
    
    // Add the new tag name if not already present
    if !current_tags.contains(&tag_name.to_string()) {
        current_tags.push(tag_name.to_string());
        println!("✅ Added tag '{}'", tag_name);
    } else {
        println!("ℹ️ Tag '{}' already exists", tag_name);
    }
    
    println!("📋 New tags: {:?}", current_tags);
    
    // Convert back to JSON
    let new_tags_json = tags_to_json(&current_tags);
    println!("📋 New tags JSON: {:?}", new_tags_json);
    
    // Update the entry
    let update = UpdateClipboardEntry {
        tags: new_tags_json,
        is_pinned: None,
    };
    
    let result = Self::update_entry(pool, clipboard_entry_id, update).await
        .map_err(|e| format!("Update failed: {}", e))?;
    
    println!("✅ Success! Updated entry tags: {:?}", json_to_tags(&result.tags));
    println!("=== ASSIGN TAG DEBUG END ===");
    
    Ok(result)
}

pub async fn remove_tag(
    pool: &PgPool, 
    clipboard_entry_id: i64, 
    tag_name: &str
) -> Result<ClipboardEntry, String> {
    println!("=== REMOVE TAG DEBUG ===");
    println!("🔴 Removing tag '{}' from entry {}", tag_name, clipboard_entry_id);
    
    // First get the current entry WITH ORGANIZATION CHECK
    let organization_id = crate::session::get_current_organization_id()
        .ok_or("User not logged in".to_string())?;
    
    let current_entry = sqlx::query_as::<_, ClipboardEntry>(
        "SELECT * FROM clipboard_entries WHERE id = $1 AND organization_id = $2"
    )
    .bind(clipboard_entry_id)
    .bind(&organization_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?
    .ok_or("Clipboard entry not found".to_string())?;
    
    println!("📋 Current entry tags: {:?}", current_entry.tags);
    
    // Parse current tags and remove the specified tag name
    let mut current_tags = json_to_tags(&current_entry.tags);
    println!("📋 Parsed current tags: {:?}", current_tags);
    
    let before_count = current_tags.len();
    current_tags.retain(|name| name != tag_name);
    let after_count = current_tags.len();
    
    println!("📋 Tags before: {}, after: {}", before_count, after_count);
    println!("📋 New tags: {:?}", current_tags);
    
    // Convert back to JSON
    let new_tags_json = tags_to_json(&current_tags);
    println!("📋 New tags JSON: {:?}", new_tags_json);
    
    // Update the entry WITH ORGANIZATION CHECK
    let result = sqlx::query_as::<_, ClipboardEntry>(
        r#"
        UPDATE clipboard_entries 
        SET tags = $1
        WHERE id = $2 AND organization_id = $3
        RETURNING *
        "#
    )
    .bind(&new_tags_json)
    .bind(clipboard_entry_id)
    .bind(&organization_id)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Update failed: {}", e))?;
    
    println!("✅ Database update successful!");
    println!("✅ Updated entry tags from query: {:?}", result.tags);
    
    // Verify with the same organization check
    let verified_entry = sqlx::query_as::<_, ClipboardEntry>(
        "SELECT * FROM clipboard_entries WHERE id = $1 AND organization_id = $2"
    )
    .bind(clipboard_entry_id)
    .bind(&organization_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Verification failed: {}", e))?
    .ok_or("Could not verify update - entry not found".to_string())?;
    
    let verified_tags = json_to_tags(&verified_entry.tags);
    println!("✅ Verified tags after update: {:?}", verified_tags);
    println!("✅ Raw verified tags: {:?}", verified_entry.tags);
    
    println!("=== REMOVE TAG DEBUG END ===");
    Ok(result)
}

}