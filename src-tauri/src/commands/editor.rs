// src-tauri/src/notepad_editor.rs
use std::{
    env, fs, io::Write,
    path::PathBuf, process::Command,
    time::{Duration, SystemTime},
};
use tokio::time::sleep;

#[tauri::command]
pub async fn open_in_notepad_and_wait(content: String) -> Result<String, String> {
    // 1. File Preparation
    let temp_file_path = create_temp_file(&content)?;
    println!("Temporary file created: {}", temp_file_path.display());

    // 2. Capture original state
    let original_metadata = capture_file_metadata(&temp_file_path)?;
    let original_content = content;

    // 3. Open in Notepad
    println!("Opening Notepad with content...");
    open_in_notepad(&temp_file_path)?;

    // 4. Wait for user editing
    println!(" Waiting for user modifications...");
    let edited_content = wait_for_edits(&temp_file_path, &original_metadata, &original_content).await;

    Ok(edited_content)
}


// Helper function: Create temporary file
fn create_temp_file(content: &str) -> Result<PathBuf, String> {
    let mut temp_dir = env::temp_dir();
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    
    temp_dir.push(format!("cliptray_edit_{}.txt", timestamp));
    
    // Write content to file
    let mut file = fs::File::create(&temp_dir)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    
    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write to temp file: {}", e))?;
    
    file.sync_all()
        .map_err(|e| format!("Failed to sync temp file: {}", e))?;
    
    // Verify file was created
    if !temp_dir.exists() {
        return Err("Temp file was not created successfully".to_string());
    }
    
    Ok(temp_dir)
}

// Helper function: Capture file metadata
fn capture_file_metadata(file_path: &PathBuf) -> Result<SystemTime, String> {
    fs::metadata(file_path)
        .and_then(|m| m.modified())
        .map_err(|e| format!("Failed to get file metadata: {}", e))
}

// Helper function: Open file in Notepad
fn open_in_notepad(file_path: &PathBuf) -> Result<(), String> {
    Command::new("notepad.exe")
        .arg(file_path)
        .spawn()
        .map_err(|e| format!("Failed to open Notepad: {}", e))?;
    
    Ok(())
}

// Helper function: Wait for and detect edits
async fn wait_for_edits(
    file_path: &PathBuf,
    original_modified: &SystemTime,
    original_content: &str,
) -> String {
    let max_attempts = 300; // 6 seconds total (30 * 200ms)
    let check_interval = Duration::from_millis(200);
    
    for attempt in 0..max_attempts {
        sleep(check_interval).await;
        
        match check_for_modifications(file_path, original_modified, original_content) {
            EditResult::Modified(new_content) => {
                println!("Content modified by user");
                return new_content;
            }
            EditResult::NotepadStillOpen => {
                // Notepad is still running, continue waiting
                if attempt % 10 == 0 { // Log every 2 seconds
                    println!("Notepad still open, waiting... (attempt {})", attempt + 1);
                }
                continue;
            }
            EditResult::NoChanges => {
                // Notepad closed but no changes detected
                if attempt < max_attempts - 1 {
                    // Wait a bit more to ensure file is fully closed
                    continue;
                } else {
                    println!("ℹ️  No modifications detected, returning original content");
                    return original_content.to_string();
                }
            }
            EditResult::Error(e) => {
                eprintln!("⚠️  Error checking file: {}", e);
                if attempt >= max_attempts - 1 {
                    return original_content.to_string();
                }
            }
        }
    }
    
    println!("⏰ Timeout reached, returning original content");
    original_content.to_string()
}

// Edit result enum for better state management
enum EditResult {
    Modified(String),
    NotepadStillOpen,
    NoChanges,
    Error(String),
}

// Helper function: Check for file modifications
fn check_for_modifications(
    file_path: &PathBuf,
    original_modified: &SystemTime,
    original_content: &str,
) -> EditResult {
    // First, check if Notepad is still running by trying to read the file
    let current_content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            // File is still locked by Notepad
            return EditResult::NotepadStillOpen;
        }
        Err(e) => {
            return EditResult::Error(format!("Failed to read file: {}", e));
        }
    };

    // Check modification timestamp
    let current_modified = match fs::metadata(file_path).and_then(|m| m.modified()) {
        Ok(modified) => modified,
        Err(e) => {
            return EditResult::Error(format!("Failed to get modification time: {}", e));
        }
    };

    // Compare content and modification time
    if current_modified > *original_modified && current_content.trim() != original_content.trim() {
        EditResult::Modified(current_content)
    } else if current_modified > *original_modified {
        // File was touched but content is the same
        EditResult::NoChanges
    } else {
        EditResult::NoChanges
    }
}