use std::sync::RwLock;
use serde::{Serialize, Deserialize};
use once_cell::sync::Lazy;

pub static CURRENT_USER: Lazy<RwLock<Option<UserSession>>> = Lazy::new(|| RwLock::new(None));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: String,
    pub organization_id: String,
    pub email: String,
}

pub fn set_current_user(user_id: String, organization_id: String, email: String) {
    let session = UserSession {
        user_id: user_id.clone(),        // Clone here
        organization_id: organization_id.clone(), // Clone here
        email: email.clone(),            // Clone here
    };
    
    if let Ok(mut current_user) = CURRENT_USER.write() {
        *current_user = Some(session);
        println!("👤 User session set - User: {}, Organization: {}", user_id, organization_id);
    } else {
        println!("❌ Failed to set user session - write lock poisoned");
    }
}

pub fn get_current_user_id() -> Option<String> {
    match CURRENT_USER.read() {
        Ok(current_user) => {
            current_user.as_ref().map(|user| user.user_id.clone())
        }
        Err(_) => {
            println!("❌ Failed to get user ID - read lock poisoned");
            None
        }
    }
}

pub fn get_current_organization_id() -> Option<String> {
    match CURRENT_USER.read() {
        Ok(current_user) => {
            current_user.as_ref().map(|user| user.organization_id.clone())
        }
        Err(_) => {
            println!("❌ Failed to get organization ID - read lock poisoned");
            None
        }
    }
}

pub fn get_current_user_email() -> Option<String> {
    match CURRENT_USER.read() {
        Ok(current_user) => {
            current_user.as_ref().map(|user| user.email.clone())
        }
        Err(_) => {
            println!("❌ Failed to get user email - read lock poisoned");
            None
        }
    }
}

pub fn get_current_session() -> Option<UserSession> {
    match CURRENT_USER.read() {
        Ok(current_user) => {
            current_user.as_ref().cloned()
        }
        Err(_) => {
            println!("❌ Failed to get session - read lock poisoned");
            None
        }
    }
}

pub fn clear_current_user() {
    if let Ok(mut current_user) = CURRENT_USER.write() {
        *current_user = None;
        println!("👤 User session cleared");
    } else {
        println!("❌ Failed to clear user session - write lock poisoned");
    }
}

// Helper function to check if user is logged in
pub fn is_user_logged_in() -> bool {
    get_current_user_id().is_some()
}