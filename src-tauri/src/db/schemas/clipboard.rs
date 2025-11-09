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

// ADD THE TAG HANDLING METHODS RIGHT HERE - INSIDE THE ClipboardEntry IMPL BLOCK
impl ClipboardEntry {
    // Get tags as Vec<String> - properly parsed from JSON
    pub fn get_tags(&self) -> Vec<String> {
        match &self.tags {
            Some(tags_json) if !tags_json.trim().is_empty() => {
                // Clean the JSON string first
                let cleaned_json = tags_json
                    .trim()
                    .replace("\\\"", "\"")  // Remove escape sequences
                    .replace("\\\\", "\\");
                
                // Parse as JSON array
                match serde_json::from_str::<Vec<String>>(&cleaned_json) {
                    Ok(tags) => tags,
                    Err(_) => {
                        // If JSON parsing fails, try to extract tags manually
                        if cleaned_json.starts_with('[') && cleaned_json.ends_with(']') {
                            let inner = &cleaned_json[1..cleaned_json.len()-1];
                            inner.split(',')
                                .map(|s| s.trim().trim_matches('"').to_string())
                                .filter(|s| !s.is_empty())
                                .collect()
                        } else {
                            // Treat as single tag
                            vec![cleaned_json.to_string()]
                        }
                    }
                }
            }
            _ => Vec::new(), // Return empty vec for None or empty strings
        }
    }
    
    // Set tags from Vec<String> - properly serialize to JSON
    pub fn set_tags(&mut self, tags: Vec<String>) {
        if tags.is_empty() {
            self.tags = None;
        } else {
            // Properly serialize to JSON array string
            self.tags = Some(serde_json::to_string(&tags).unwrap_or_else(|_| {
                // Fallback: manual JSON creation
                let tags_json = tags
                    .iter()
                    .map(|tag| format!("\"{}\"", tag.replace('\"', "\\\"")))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("[{}]", tags_json)
            }));
        }
    }
    
    // Add a single tag
    pub fn add_tag(&mut self, tag: String) {
        let mut current_tags = self.get_tags();
        if !current_tags.contains(&tag) {
            current_tags.push(tag);
            self.set_tags(current_tags);
        }
    }
    
    // Remove a tag by name
    pub fn remove_tag(&mut self, tag_name: &str) {
        let mut current_tags = self.get_tags();
        current_tags.retain(|t| t != tag_name);
        self.set_tags(current_tags);
    }
    
    // Check if entry has a specific tag
    pub fn has_tag(&self, tag_name: &str) -> bool {
        self.get_tags().iter().any(|t| t == tag_name)
    }
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