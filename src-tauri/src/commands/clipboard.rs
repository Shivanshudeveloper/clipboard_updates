// src-tauri/src/clipboard.rs
use arboard::Clipboard;
use tokio::time;
use std::time::Duration;
use tauri::AppHandle;
use tauri::Emitter;
use serde::Serialize;
use crate::db::{ClipboardRepository, NewClipboardEntry};
use sqlx::PgPool;

// Configuration
const POLL_INTERVAL_MS: u64 = 1000;

#[derive(Clone, Serialize)]
pub struct ClipboardContent {
    pub text: String,
    pub timestamp: u64,
    pub content_type: String,
    pub source_app: String,
    pub source_window: String,
}

#[cfg(target_os = "windows")]
pub fn get_foreground_window_info() -> Option<(String, String)> {
    use windows::{
        Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowTextLengthW},
        Win32::Foundation::HWND,
    };

    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        
        // FIXED: Proper null pointer check
        if hwnd.0.is_null() {
            return None;
        }

        let length = GetWindowTextLengthW(hwnd);
        if length == 0 {
            return None;
        }

        let mut buffer = vec![0u16; (length + 1) as usize];
        let actual_length = GetWindowTextW(hwnd, &mut buffer);
        
        if actual_length == 0 {
            return None;
        }

        let window_title = String::from_utf16_lossy(&buffer[..actual_length as usize]);
        
        // Extract app name from window title
        let app_name = extract_app_name_from_title(&window_title);
        
        Some((app_name, window_title))
    }
}

#[cfg(target_os = "windows")]
fn extract_app_name_from_title(title: &str) -> String {
    // Simple heuristic to extract app name from window title
    if title.contains(" - ") {
        if let Some(app_part) = title.split(" - ").last() {
            return app_part.to_string();
        }
    }
    
    // Fallback: use the first few words or truncate
    if title.len() > 20 {
        format!("{}...", &title[..17])
    } else {
        title.to_string()
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_foreground_window_info() -> Option<(String, String)> {
    // Default implementation for non-Windows platforms
    None
}

pub async fn start_clipboard_monitoring(
    app_handle: AppHandle,
    db_pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut clipboard = Clipboard::new()?;
    let mut last_content = String::new();
    
    println!("🔍 Clipboard monitoring started with window detection...");

    loop {
        time::sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
        
        // Get foreground window info before checking clipboard
        let window_info = get_foreground_window_info();
        let (source_app, source_window) = window_info.unwrap_or_else(|| {
            ("Unknown".to_string(), "Unknown".to_string())
        });

        match clipboard.get_text() {
            Ok(content) => {
                if !content.trim().is_empty() && content != last_content {
                    println!("📋 Clipboard text: '{}'", content);
                    println!("📍 Source: '{}'", source_window);
                    
                    // ✅ Get the organization ID from the user session
                    let organization_id = crate::session::get_current_organization_id();
                    
                    if organization_id.is_none() {
                        println!("⚠️ No organization ID found - user not logged in, skipping clipboard save");
                        last_content = content;
                        continue;
                    }
                    
                    let org_id = organization_id.unwrap();
                    println!("🏢 Organization ID for clipboard entry: {}", org_id);
                    
                    // Create the clipboard entry
                    let mut new_entry = NewClipboardEntry::from_monitoring_data(
                        content.clone(),
                        source_app.clone(),
                        source_window.clone(),
                    );
                    
                    // ✅ Set the organization ID for the clipboard entry
                    new_entry.organization_id = Some(org_id.clone());
                    
                    // Save to database
                    match ClipboardRepository::save_entry(&db_pool, new_entry).await {
                        Ok(saved_entry) => {
                            println!("✅ Saved clipboard entry #{} for organization: {}", 
                                     saved_entry.id, org_id);
                        }
                        Err(e) => {
                            println!("❌ Failed to save clipboard entry: {}", e);
                        }
                    }

                    let clipboard_content = ClipboardContent {
                        text: content.clone(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        content_type: "text".to_string(),
                        source_app: source_app.clone(),
                        source_window: source_window.clone(),
                    };
                    
                    if let Err(e) = app_handle.emit("clipboard-update", &clipboard_content) {
                        println!("❌ Failed to emit clipboard event: {}", e);
                    }
                    
                    last_content = content;
                }
            }
            Err(arboard::Error::ContentNotAvailable) => {
                // Clipboard doesn't contain text content, this is normal
            }
            Err(e) => {
                eprintln!("⚠️ Clipboard error: {}", e);
                // Try to reinitialize clipboard on certain errors
                if let Ok(new_clipboard) = Clipboard::new() {
                    clipboard = new_clipboard;
                    println!("🔄 Clipboard reinitialized");
                }
            }
        }
    }
}