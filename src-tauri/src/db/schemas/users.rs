use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use serde_json::Value;
use chrono::{DateTime, Utc};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "purge_cadence", rename_all = "snake_case")]
pub enum PurgeCadence {
    Never,
    #[sqlx(rename = "every_24_hours")]
    Every24Hours,
    #[sqlx(rename = "every_3_days")]
    Every3Days,
    #[sqlx(rename = "every_week")]
    EveryWeek,
    #[sqlx(rename = "every_month")]
    EveryMonth,
}

impl Default for PurgeCadence {
    fn default() -> Self {
        Self::Never
    }

    
}

impl FromStr for PurgeCadence {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "never" => Ok(Self::Never),
            "every_24_hours" | "24h" | "24hours" | "every 24 hours" => Ok(Self::Every24Hours),
            "every_3_days" | "3d" | "3days" | "every 3 days" => Ok(Self::Every3Days),
            "every_week" | "7d" | "7days" | "every week" | "weekly" => Ok(Self::EveryWeek),
            "every_month" | "30d" | "30days" | "every month" | "monthly" => Ok(Self::EveryMonth),
            _ => Err(format!("Invalid purge cadence: {}", s)),
        }
    }
}

impl PurgeCadence {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Never => "never",
            Self::Every24Hours => "every_24_hours",
            Self::Every3Days => "every_3_days",
            Self::EveryWeek => "every_week",
            Self::EveryMonth => "every_month",
        }
    }

    pub fn to_display_string(&self) -> &'static str {
        match self {
            Self::Never => "Never",
            Self::Every24Hours => "Every 24 hours",
            Self::Every3Days => "Every 3 days",
            Self::EveryWeek => "Every week",
            Self::EveryMonth => "Every month",
        }
    }

    pub fn from_display_string(s: &str) -> Result<Self, String> {
        match s {
            "Never" => Ok(Self::Never),
            "Every 24 hours" => Ok(Self::Every24Hours),
            "Every 3 days" => Ok(Self::Every3Days),
            "Every week" => Ok(Self::EveryWeek),
            "Every month" => Ok(Self::EveryMonth),
            _ => Err(format!("Invalid purge cadence display string: {}", s)),
        }
    }

    pub fn to_duration(&self) -> Option<chrono::Duration> {
        match self {
            Self::Never => None,
            Self::Every24Hours => Some(chrono::Duration::hours(24)),
            Self::Every3Days => Some(chrono::Duration::days(3)),
            Self::EveryWeek => Some(chrono::Duration::days(7)),
            Self::EveryMonth => Some(chrono::Duration::days(30)),
        }
    }

    pub fn all_options() -> Vec<&'static str> {
        vec![
            "Never",
            "Every 24 hours", 
            "Every 3 days",
            "Every week",
            "Every month",
        ]
    }

    pub fn to_days_i32(&self) -> Option<i32> {
        self.to_duration().map(|d| {
            let days = d.num_days();
            if days <= 0 {
                1
            } else {
                days as i32
            }
        })
    }
    
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub firebase_uid: String,
    pub email: String,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub organization_id: Option<String>,
    pub purge_cadence: PurgeCadence, // New field with default Never
    pub retain_tags: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub firebase_uid: String,
    pub email: String,
    pub display_name: Option<String>,
    pub organization_id: Option<String>,
    // No purge_cadence here - it will always default to Never for new users
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateUser {
    pub display_name: Option<String>,
    pub purge_cadence: Option<PurgeCadence>, // Can be updated via settings
}

// Response DTOs for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i64,
    pub firebase_uid: String,
    pub email: String,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub organization_id: Option<String>,
    pub purge_cadence: String, // Serialized as display string for frontend
    pub retain_tags: bool,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            firebase_uid: user.firebase_uid,
            email: user.email,
            display_name: user.display_name,
            created_at: user.created_at,
            organization_id: user.organization_id,
            purge_cadence: user.purge_cadence.to_display_string().to_string(),
            retain_tags: user.retain_tags,
        }
    }
}

impl User {
    // Helper to get default preferences as Value
    pub fn default_preferences() -> Value {
        serde_json::json!({
            "theme": "system",
            "language": "en", 
            "clipboard_retention_days": 30,
            "max_clipboard_entries": 1000,
            "auto_clear_interval": null,
            "sync_enabled": true,
            "keyboard_shortcuts": null
        })
    }
}