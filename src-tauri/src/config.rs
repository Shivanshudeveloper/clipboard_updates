// config.rs
use lazy_static::lazy_static;
use std::sync::RwLock;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub github_owner: &'static str,
    pub github_repo: &'static str,
    pub current_version: &'static str,
    pub database_url: &'static str,
    pub firebase_project_id: &'static str,
    pub client_id: &'static str,
    pub client_secret: &'static str,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            github_owner: "Shivanshudeveloper",
            github_repo: "clipboard_updates",
            current_version: "0.2.6",
            database_url: "DATABASE_URL",
            firebase_project_id: "PROJECT_ID",
            client_id: "YOUR_CLIENT_ID",
            client_secret: "YOUR_CLIENT_SECRET",
        }
    }
}

lazy_static! {
    pub static ref CONFIG: RwLock<AppConfig> = RwLock::new(AppConfig::default());
}

// Helper functions to access config - now return &'static str
pub fn get_github_owner() -> &'static str {
    CONFIG.read().unwrap().github_owner
}

pub fn get_github_repo() -> &'static str {
    CONFIG.read().unwrap().github_repo
}

pub fn get_current_version() -> &'static str {
    CONFIG.read().unwrap().current_version
}

pub fn get_database_url() -> &'static str {
    CONFIG.read().unwrap().database_url
}

pub fn get_firebase_project_id() -> &'static str {
    CONFIG.read().unwrap().firebase_project_id
}


pub fn get_client_id() -> &'static str {
    CONFIG.read().unwrap().client_id
}


pub fn get_client_secret() -> &'static str {
    CONFIG.read().unwrap().client_secret
}