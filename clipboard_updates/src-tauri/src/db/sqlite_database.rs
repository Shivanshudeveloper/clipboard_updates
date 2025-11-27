// src/db/sqlite_database.rs
use sqlx::{sqlite::{SqlitePool, SqlitePoolOptions}};
use chrono::Utc;

// Reuse your existing schemas from database.rs
use crate::db::schemas::{ClipboardEntry, NewClipboardEntry, UpdateClipboardEntry};

pub async fn create_sqlite_pool() -> Result<SqlitePool, Box<dyn std::error::Error>> {   
    // Use a local SQLite database file
    let database_url = "sqlite://cliptray_offline.db?mode=rwc";
    println!("Connecting to SQLite database...");
    
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await?;    
    println!("SQLite database connected successfully!");    
    // Create tables if they don't exist
    create_sqlite_tables(&pool).await?;    
    Ok(pool)
}

pub async fn create_sqlite_tables(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Creating SQLite database tables if they don't exist...");

    println!("📝 Creating Users table if not exists...");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            organization_id TEXT NOT NULL,
            firebase_uid TEXT UNIQUE NOT NULL,
            email TEXT NOT NULL,
            display_name TEXT,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            purge_cadence TEXT NOT NULL DEFAULT 'never',
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_login_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
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
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            organization_id TEXT NOT NULL,
            content TEXT NOT NULL,
            content_type TEXT NOT NULL DEFAULT 'text',
            content_hash TEXT UNIQUE NOT NULL,
            source_app TEXT NOT NULL,
            source_window TEXT NOT NULL,
            timestamp DATETIME NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            tags TEXT,
            is_pinned BOOLEAN NOT NULL DEFAULT FALSE,
            sync_status TEXT NOT NULL DEFAULT 'local',
            server_id TEXT
        )
        "#
    )
    .execute(pool)
    .await?;

    println!("📝 Creating Tags table if not exists...");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            organization_id TEXT NOT NULL,
            name TEXT NOT NULL,
            color TEXT NOT NULL DEFAULT '#6B7280',
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
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
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_clipboard_sync_status ON clipboard_entries(sync_status)")
        .execute(pool).await?;

    // Tags indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_tags_organization_id ON tags(organization_id)")
        .execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name)")
        .execute(pool).await?;
    sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS idx_tags_organization_name_unique ON tags(organization_id, name)")
        .execute(pool).await?;
    
    println!("✅ SQLite database tables ready!");
    Ok(())
}

// Import the helper functions from your existing database.rs
use crate::db::database::{tags_to_json, json_to_tags};

// SQLite Clipboard operations
pub struct SqliteClipboardRepository;

impl SqliteClipboardRepository {
    
    pub async fn save_entry(
        pool: &SqlitePool, 
        entry: NewClipboardEntry
    ) -> Result<ClipboardEntry, Box<dyn std::error::Error>> {
        let result = sqlx::query_as::<_, ClipboardEntry>(
            r#"
            INSERT INTO clipboard_entries 
            (content, content_type, content_hash, source_app, source_window, timestamp, tags, organization_id, sync_status)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'local')
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
        .fetch_one(pool)
        .await?;
        
        Ok(result)
    }

    pub async fn get_by_organization(
        pool: &SqlitePool, 
        organization_id: &str,
        limit: Option<i64>
    ) -> Result<Vec<ClipboardEntry>, Box<dyn std::error::Error>> {
        let limit = limit.unwrap_or(100);
        
        let results = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries WHERE organization_id = ?1 ORDER BY created_at DESC LIMIT ?2"
        )
        .bind(organization_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;
        
        Ok(results)
    }
    
    pub async fn get_by_id(
        pool: &SqlitePool, 
        id: i64
    ) -> Result<Option<ClipboardEntry>, Box<dyn std::error::Error>> {
        let result = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries WHERE id = ?1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        
        Ok(result)
    }
    
    pub async fn get_all(
        pool: &SqlitePool, 
        limit: Option<i64>
    ) -> Result<Vec<ClipboardEntry>, Box<dyn std::error::Error>> {
        let limit = limit.unwrap_or(100);
        
        let results = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries ORDER BY created_at DESC LIMIT ?1"
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;
        
        Ok(results)
    }
    
    pub async fn get_recent(
        pool: &SqlitePool, 
        hours: i32
    ) -> Result<Vec<ClipboardEntry>, Box<dyn std::error::Error>> {
        let results = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries WHERE created_at > datetime('now', ?1) ORDER BY created_at DESC"
        )
        .bind(format!("-{} hours", hours))
        .fetch_all(pool)
        .await?;
        
        Ok(results)
    }
    
    pub async fn search_content(
        pool: &SqlitePool, 
        query: &str
    ) -> Result<Vec<ClipboardEntry>, Box<dyn std::error::Error>> {
        let search_pattern = format!("%{}%", query);
        
        let results = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries WHERE content LIKE ?1 ORDER BY created_at DESC"
        )
        .bind(search_pattern)
        .fetch_all(pool)
        .await?;
        
        Ok(results)
    }
    
    pub async fn update_entry(
        pool: &SqlitePool, 
        id: i64, 
        update: UpdateClipboardEntry
    ) -> Result<ClipboardEntry, Box<dyn std::error::Error>> {
        let result = sqlx::query_as::<_, ClipboardEntry>(
            r#"
            UPDATE clipboard_entries 
            SET 
                is_pinned = COALESCE(?1, is_pinned),
                tags = COALESCE(?2, tags)
            WHERE id = ?3
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
        pool: &SqlitePool, 
        id: i64
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let result = sqlx::query(
            "DELETE FROM clipboard_entries WHERE id = ?1"
        )
        .bind(id)
        .execute(pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }

    pub async fn update_entry_content(
        pool: &SqlitePool,
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
                content = ?1,
                content_hash = ?2,
                timestamp = ?3
            WHERE id = ?4
            RETURNING *
            "#
        )
        .bind(new_content)
        .bind(content_hash)
        .bind(Utc::now())
        .bind(entry_id)
        .fetch_one(pool)
        .await?;
        
        Ok(result)
    }
    
    pub async fn exists_by_hash(
        pool: &SqlitePool, 
        content_hash: &str
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let result = sqlx::query(
            "SELECT 1 FROM clipboard_entries WHERE content_hash = ?1 LIMIT 1"
        )
        .bind(content_hash)
        .fetch_optional(pool)
        .await?;
        
        Ok(result.is_some())
    }

    // Settings commands
    pub async fn delete_entries_older_than(pool: &SqlitePool, organization_id: &str, days: i32) -> Result<usize, sqlx::Error> {
        sqlx::query(
            "DELETE FROM clipboard_entries WHERE organization_id = ?1 AND created_at < datetime('now', ?2)"
        )
        .bind(organization_id)
        .bind(format!("-{} days", days))
        .execute(pool)
        .await
        .map(|result| result.rows_affected() as usize)
    }
    
    pub async fn delete_unpinned_older_than(pool: &SqlitePool, organization_id: &str, days: i32) -> Result<usize, sqlx::Error> {
        sqlx::query(
            "DELETE FROM clipboard_entries WHERE organization_id = ?1 AND is_pinned = false AND created_at < datetime('now', ?2)"
        )
        .bind(organization_id)
        .bind(format!("-{} days", days))
        .execute(pool)
        .await
        .map(|result| result.rows_affected() as usize)
    }

    pub async fn delete_untagged_entries(pool: &SqlitePool, organization_id: &str) -> Result<usize, sqlx::Error> {
        sqlx::query(
            "DELETE FROM clipboard_entries WHERE organization_id = ?1 AND is_pinned = false AND tags IS NULL"
        )
        .bind(organization_id)
        .execute(pool)
        .await
        .map(|result| result.rows_affected() as usize)
    }

    pub async fn delete_unpinned_entries(pool: &SqlitePool, organization_id: &str) -> Result<usize, sqlx::Error> {
        sqlx::query(
            "DELETE FROM clipboard_entries WHERE organization_id = ?1 AND is_pinned = false"
        )
        .bind(organization_id)
        .execute(pool)
        .await
        .map(|result| result.rows_affected() as usize)
    }

    pub async fn assign_tag(
        pool: &SqlitePool, 
        clipboard_entry_id: i64, 
        tag_name: &str
    ) -> Result<ClipboardEntry, String> {
        println!("=== ASSIGN TAG DEBUG (SQLite) ===");
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
        pool: &SqlitePool, 
        clipboard_entry_id: i64, 
        tag_name: &str
    ) -> Result<ClipboardEntry, String> {
        println!("=== REMOVE TAG DEBUG (SQLite) ===");
        println!("🔴 Removing tag '{}' from entry {}", tag_name, clipboard_entry_id);
        
        // For offline mode, we don't need organization check
        let current_entry = Self::get_by_id(pool, clipboard_entry_id).await
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
        
        // Convert back to JSON - FIXED: use tags_to_json instead of tags_to_tags
        let new_tags_json = tags_to_json(&current_tags);
        println!("📋 New tags JSON: {:?}", new_tags_json);
        
        // Update the entry
        let result = sqlx::query_as::<_, ClipboardEntry>(
            r#"
            UPDATE clipboard_entries 
            SET tags = ?1
            WHERE id = ?2
            RETURNING *
            "#
        )
        .bind(&new_tags_json)
        .bind(clipboard_entry_id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Update failed: {}", e))?;
        
        println!("✅ Database update successful!");
        println!("✅ Updated entry tags from query: {:?}", result.tags);
        
        // Verify the update
        let verified_entry = Self::get_by_id(pool, clipboard_entry_id).await
            .map_err(|e| format!("Verification failed: {}", e))?
            .ok_or("Could not verify update - entry not found".to_string())?;
        
        let verified_tags = json_to_tags(&verified_entry.tags);
        println!("✅ Verified tags after update: {:?}", verified_tags);
        println!("✅ Raw verified tags: {:?}", verified_entry.tags);
        
        println!("=== REMOVE TAG DEBUG END ===");
        Ok(result)
    }

    // Additional offline-specific methods
    pub async fn get_pending_sync_entries(pool: &SqlitePool) -> Result<Vec<ClipboardEntry>, Box<dyn std::error::Error>> {
        let results = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries WHERE sync_status = 'local' ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await?;
        
        Ok(results)
    }

    pub async fn mark_as_synced(pool: &SqlitePool, local_id: i64, server_id: i64) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query(
            "UPDATE clipboard_entries SET sync_status = 'synced', server_id = ?1 WHERE id = ?2"
        )
        .bind(server_id.to_string())
        .bind(local_id)
        .execute(pool)
        .await?;
        
        Ok(())
    }
}