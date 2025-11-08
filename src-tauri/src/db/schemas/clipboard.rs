use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ClipboardEntry {
    pub id: i64,
    pub content: String,
    pub content_type: String,        // "text", "image", "file", etc.
    pub content_hash: String,        // For deduplication
    pub source_app: String,
    pub source_window: String,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,    
    pub tags: Option<String>,        // JSON array of tags
    pub is_pinned: bool,
    pub organization_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewClipboardEntry {
    pub content: String,
    pub content_type: String,
    pub content_hash: String,
    pub source_app: String,
    pub source_window: String,
    pub timestamp: DateTime<Utc>,
    pub tags: Option<String>,
    pub organization_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateClipboardEntry {
    pub is_pinned: Option<bool>,
    pub tags: Option<String>,
}

// In src/db/schemas/clipboard.rs
impl NewClipboardEntry {
    pub fn from_monitoring_data(
        content: String,
        source_app: String,
        source_window: String,
    ) -> Self {
        let content_hash = format!("{:x}", md5::compute(&content));
        let content_type = detect_content_type(&content);
        
        Self {
            content,
            content_type,
            content_hash,
            source_app,
            source_window,
            timestamp: Utc::now(),
            tags: None,
            organization_id:None, // Set to None initially
        }
    }
}

fn detect_content_type(content: &str) -> String {
    if content.starts_with("http://") || content.starts_with("https://") {
        "url".to_string()
    } else if content.contains('@') && content.contains('.') {
        "email".to_string()
    } else if content.chars().all(|c| c.is_numeric() || c.is_whitespace()) {
        "numeric".to_string()
    } else {
        "text".to_string()
    }
}