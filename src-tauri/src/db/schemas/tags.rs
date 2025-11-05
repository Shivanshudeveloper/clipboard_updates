// src/models/tag.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: i64,
    pub organization_id: String,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTag {
    pub organization_id: String,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateTag {
    pub name: Option<String>,
    pub color: Option<String>,
}

impl Tag {
    // Validate tag name
    pub fn is_valid_name(name: &str) -> bool {
        let trimmed = name.trim();
        !trimmed.is_empty() && trimmed.len() <= 50
    }

    // Validate color format (hex color)
    pub fn is_valid_color(color: &str) -> bool {
        let color = color.trim();
        (color.starts_with('#') && color.len() == 7) || 
        (!color.starts_with('#') && color.len() == 6)
    }

    // Format color to always include #
    pub fn format_color(color: &str) -> String {
        let color = color.trim();
        if color.starts_with('#') {
            color.to_string()
        } else {
            format!("#{}", color)
        }
    }
}

// Response DTOs for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagResponse {
    pub id: i64,
    pub organization_id: String,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Tag> for TagResponse {
    fn from(tag: Tag) -> Self {
        Self {
            id: tag.id,
            organization_id: tag.organization_id,
            name: tag.name,
            color: tag.color,
            created_at: tag.created_at,
            updated_at: tag.updated_at,
        }
    }
}

// For bulk operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkTagOperation {
    pub organization_id: String,
    pub tags: Vec<NewTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagStats {
    pub tag_id: i64,
    pub tag_name: String,
    pub usage_count: i64,
    pub last_used_at: Option<DateTime<Utc>>,
}