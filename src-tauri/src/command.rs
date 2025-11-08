use tauri::command;
use crate::db::schemas::ClipboardEntry;
use crate::auth::verify_firebase_token;
use crate::db::database::create_db_pool;
use crate::db::users_repository::UsersRepository;
use crate::db::schemas::users::{NewUser, UserResponse,PurgeCadence};
// use crate::db::schemas::users::PurgeCadence;
use crate::db::schemas::tags::{Tag, NewTag, UpdateTag, TagResponse};
use crate::db::tags_repository::TagRepository;
use rand::Rng;
use serde_json;
use tauri::{State, Manager}; // Add these imports
use sqlx::PgPool;
use tauri_plugin_updater::UpdaterExt;
use std::time::Duration;
use tauri::Emitter;
use tauri::AppHandle;
use tauri::async_runtime::Mutex;
use crate::updater::{Updater, UpdateCheckResult,  InstallerInfo};

#[tauri::command]
pub async fn debug_get_specific_fields() -> serde_json::Value {
    serde_json::json!({
        "user_id": crate::session::get_current_user_id(),
        "organization_id": crate::session::get_current_organization_id(),
        "email": crate::session::get_current_user_email(),
        "is_logged_in": crate::session::is_user_logged_in()
    })
}

// ALL commands should use the managed pool state
#[command]
pub async fn get_all_entries(
    limit: Option<i64>,
    pool: tauri::State<'_, sqlx::PgPool>
) -> Result<Vec<ClipboardEntry>, String> {
    crate::db::ClipboardRepository::get_all(&pool, limit)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_my_entries(
    limit: Option<i64>,
    pool: tauri::State<'_, sqlx::PgPool>
) -> Result<Vec<ClipboardEntry>, String> {
    // Get current organization ID from session
    let organization_id = crate::session::get_current_organization_id()
        .ok_or("User not logged in".to_string())?;
    
    crate::db::ClipboardRepository::get_by_organization(&pool, &organization_id, limit)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_recent_entries(
    hours: Option<i32>,
    pool: tauri::State<'_, sqlx::PgPool>
) -> Result<Vec<ClipboardEntry>, String> {
    let hours = hours.unwrap_or(24);
    crate::db::ClipboardRepository::get_recent(&pool, hours)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_entry_by_id(
    id: i64,
    pool: tauri::State<'_, sqlx::PgPool>
) -> Result<Option<ClipboardEntry>, String> {
    crate::db::ClipboardRepository::get_by_id(&pool, id)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn delete_entry(
    id: i64,
    pool: tauri::State<'_, sqlx::PgPool>
) -> Result<bool, String> {
    crate::db::ClipboardRepository::delete_entry(&pool, id)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn update_entry_content(
    id: i64,
    new_content: String,
    pool: tauri::State<'_, sqlx::PgPool>
) -> Result<ClipboardEntry, String> {
    crate::db::ClipboardRepository::update_entry_content(&pool, id, &new_content)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn search_entries(
    query: String,
    pool: tauri::State<'_, sqlx::PgPool>
) -> Result<Vec<ClipboardEntry>, String> {
    crate::db::ClipboardRepository::search_content(&pool, &query)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn update_entry(
    id: i64,
    updates: serde_json::Value, // Or create a proper struct
    pool: tauri::State<'_, sqlx::PgPool>
) -> Result<ClipboardEntry, String> {
    use crate::db::schemas::UpdateClipboardEntry;
    
    let update_struct = UpdateClipboardEntry {
        is_pinned: updates.get("is_pinned").and_then(|v| v.as_bool()),
        ..Default::default()
    };
    
    crate::db::ClipboardRepository::update_entry(&pool, id, update_struct)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn login_user(
    firebase_token: String,
    display_name: String,
    // Remove organization_id parameter - let backend generate it
) -> Result<UserResponse, String> {
    println!("🔐 Starting login_user command...");
    println!("👤 Received display name: {}", display_name);
    
    let (uid, email, _) = verify_firebase_token(&firebase_token).await?;
    println!("✅ Firebase UID verified: {}", uid);
    println!("📧 User email: {}", email);

    let pool = create_db_pool()
        .await
        .map_err(|e| format!("Database connection failed: {}", e))?;

    // ✅ Check if user exists by UID
    if let Some(user) = UsersRepository::get_by_firebase_uid(&pool, &uid)
        .await
        .map_err(|e| e.to_string())?
    {
        println!("🟢 Returning existing user: {}", user.email);
        
        let real_organization_id = user.organization_id.clone().unwrap_or(uid.clone());
        println!("🏢 Real organization ID from DB: {:?}", real_organization_id);
        
        // ✅ SET SESSION WITH REAL ORGANIZATION ID
        crate::session::set_current_user(
            user.firebase_uid.clone(),
            real_organization_id,
            user.email.clone(),
        );
        println!("👤 Session set for existing user");
        
        return Ok(UserResponse::from(user));
    }

    // ✅ For new users, generate a proper organization ID
    let new_organization_id = format!("org_{}", uid); // Or use a proper UUID
    
    let new_user = NewUser {
        firebase_uid: uid.clone(),
        email: email.clone(),
        display_name: Some(display_name),
        organization_id: Some(new_organization_id.clone()),
    };

    println!("📝 Creating user with generated organization ID: {}", new_organization_id);
    
    match UsersRepository::create_user(&pool, &new_user).await {
        Ok(created) => {
            println!("✅ User created successfully");
            
            // Set current user session
            crate::session::set_current_user(
                created.firebase_uid.clone(),
                new_organization_id,
                created.email.clone(),
            );
            println!("👤 Session set for new user");
            
            Ok(UserResponse::from(created))
        }
        Err(e) => {
            println!("❌ Failed to create user: {}", e);
            Err(format!("Failed to create user: {}", e))
        }
    }
}

#[command]
pub async fn signup_user(
    firebase_token: String,
    display_name: String,
    organization_id: String,
) -> Result<UserResponse, String> {
    println!("🔐 Starting signup_user command...");
    println!("👤 Received display name: {}", display_name);
    println!("🏢 Received organization ID: {}", organization_id);    
    
    let (uid, email, _) = verify_firebase_token(&firebase_token).await?;
    println!("✅ Firebase UID verified: {}", uid);
    println!("📧 User email: {}", email);

    let pool = create_db_pool()
        .await
        .map_err(|e| format!("Database connection failed: {}", e))?;

    // ✅ Check if user already exists
    if let Some(existing_user) = UsersRepository::get_by_firebase_uid(&pool, &uid)
        .await
        .map_err(|e| e.to_string())?
    {
        println!("❌ User already exists: {}", existing_user.email);
        return Err("User already exists. Please login instead.".to_string());
    }

    // ✅ Create new user
    let new_user = NewUser {
        firebase_uid: uid.clone(),
        email: email.clone(),
        display_name: Some(display_name),
        organization_id: Some(organization_id.clone()),
    };

    println!("📝 Creating user with data - Email: {}, Display Name: {:?}", 
             new_user.email, new_user.display_name);
    
    match UsersRepository::create_user(&pool, &new_user).await {
        Ok(created) => {
            println!("✅ User created successfully - ID: {}, Email: {}, Name: {:?}", 
                     created.id, created.email, created.display_name);
            
            // Set current user session
            crate::session::set_current_user(
                created.firebase_uid.clone(),
                organization_id.clone(),
                created.email.clone(),
            );
            println!("👤 Session set for new user");
            
            Ok(UserResponse::from(created))
        }
        Err(e) => {
            println!("❌ Failed to create user: {}", e);
            Err(format!("Failed to create user: {}", e))
        }
    }
}

#[command]
pub async fn logout_user() -> Result<String, String> {
    crate::session::clear_current_user();
    Ok("User logged out".to_string())
}





//Tags Commannds

#[tauri::command]
pub async fn get_organization_tags(
    pool: tauri::State<'_, sqlx::PgPool>
) -> Result<Vec<TagResponse>, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or("User not logged in".to_string())?;
    
    println!("🏢 Fetching tags for organization: {}", organization_id);
    
    let tag_repo = TagRepository::new(pool.inner().clone());
    let tags: Vec<Tag> = tag_repo.get_organization_tags(&organization_id) // Specify type
        .await
        .map_err(|e: sqlx::Error| format!("Failed to fetch tags: {}", e))?;
    
    println!("✅ Found {} tags for organization: {}", tags.len(), organization_id);
    
    Ok(tags.into_iter().map(TagResponse::from).collect())
}

#[tauri::command]
pub async fn create_tag(
    name: String,
    color: Option<String>,
    pool: tauri::State<'_, sqlx::PgPool>
) -> Result<TagResponse, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or("User not logged in".to_string())?;
    
    println!("🏢 Creating tag for organization: {}", organization_id);
    println!("📝 Tag name: {}, Color: {:?}", name, color);
    
    // Validate tag name
    if !Tag::is_valid_name(&name) {
        return Err("Tag name must be between 1 and 50 characters".to_string());
    }
    
    // Use provided color or generate random one
    let tag_color = color.unwrap_or_else(|| {
        let mut rng = rand::thread_rng();
        format!("#{:06x}", rng.gen::<u32>() & 0xFFFFFF)
    });
    
    // Validate color format
    if !Tag::is_valid_color(&tag_color) {
        return Err("Invalid color format. Use hex format like #FF0000".to_string());
    }
    
    let formatted_color = Tag::format_color(&tag_color);
    
    let tag_repo = TagRepository::new(pool.inner().clone());
    
    // Check if tag name already exists in organization
    let exists: bool = tag_repo.tag_name_exists(&organization_id, &name) // Specify type
        .await
        .map_err(|e: sqlx::Error| format!("Failed to check tag existence: {}", e))?;
    
    if exists {
        return Err(format!("Tag '{}' already exists in this organization", name));
    }
    
    let new_tag = NewTag {
        organization_id: organization_id.clone(),
        name: name.trim().to_string(),
        color: formatted_color,
    };
    
    let created_tag: Tag = tag_repo.create_tag(&new_tag) // Specify type
        .await
        .map_err(|e: sqlx::Error| format!("Failed to create tag: {}", e))?;
    
    println!("✅ Tag created successfully - ID: {}, Name: {}", created_tag.id, created_tag.name);
    
    Ok(TagResponse::from(created_tag))
}

#[tauri::command]
pub async fn update_tag(
    tag_id: i64,
    updates: serde_json::Value,
    pool: tauri::State<'_, sqlx::PgPool>
) -> Result<TagResponse, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or("User not logged in".to_string())?;
    
    println!("🏢 Updating tag {} for organization: {}", tag_id, organization_id);
    println!("📝 Updates: {:?}", updates);
    
    let tag_repo = TagRepository::new(pool.inner().clone());
    
    // Build update struct
    let mut update_struct = UpdateTag::default();
    
    if let Some(name_value) = updates.get("name") {
        if let Some(name) = name_value.as_str() {
            if !Tag::is_valid_name(name) {
                return Err("Tag name must be between 1 and 50 characters".to_string());
            }
            
            // Check if new name conflicts with existing tag
            let exists: bool = tag_repo.tag_name_exists(&organization_id, name) // Specify type
                .await
                .map_err(|e: sqlx::Error| format!("Failed to check tag existence: {}", e))?;
            
            if exists {
                // But allow if it's the same tag being updated
                let current_tag: Option<Tag> = tag_repo.get_tag(tag_id, &organization_id) // Specify type
                    .await
                    .map_err(|e: sqlx::Error| format!("Failed to get current tag: {}", e))?;
                
                if let Some(current_tag) = current_tag {
                    if current_tag.name != name {
                        return Err(format!("Tag '{}' already exists in this organization", name));
                    }
                }
            }
            
            update_struct.name = Some(name.trim().to_string());
        }
    }
    
    if let Some(color_value) = updates.get("color") {
        if let Some(color) = color_value.as_str() {
            if !Tag::is_valid_color(color) {
                return Err("Invalid color format. Use hex format like #FF0000".to_string());
            }
            update_struct.color = Some(Tag::format_color(color));
        }
    }
    
    let updated_tag: Tag = tag_repo.update_tag(tag_id, &organization_id, &update_struct) // Specify type
        .await
        .map_err(|e: sqlx::Error| format!("Failed to update tag: {}", e))?
        .ok_or("Tag not found".to_string())?;
    
    println!("✅ Tag updated successfully - ID: {}, Name: {}", updated_tag.id, updated_tag.name);
    
    Ok(TagResponse::from(updated_tag))
}

#[tauri::command]
pub async fn delete_tag(
    tag_id: i64,
    pool: tauri::State<'_, sqlx::PgPool>
) -> Result<bool, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or("User not logged in".to_string())?;
    
    println!("🏢 Deleting tag {} for organization: {}", tag_id, organization_id);
    
    let tag_repo = TagRepository::new(pool.inner().clone());
    
    // First check if tag exists and belongs to organization
    let tag: Option<Tag> = tag_repo.get_tag(tag_id, &organization_id) // Specify type
        .await
        .map_err(|e: sqlx::Error| format!("Failed to get tag: {}", e))?;
    
    let tag = tag.ok_or("Tag not found".to_string())?;
    
    println!("🗑️ Deleting tag: {} (ID: {})", tag.name, tag.id);
    
    let deleted: bool = tag_repo.delete_tag(tag_id, &organization_id) // Specify type
        .await
        .map_err(|e: sqlx::Error| format!("Failed to delete tag: {}", e))?;
    
    if deleted {
        println!("✅ Tag deleted successfully - ID: {}, Name: {}", tag_id, tag.name);
        Ok(true)
    } else {
        println!("❌ Failed to delete tag - ID: {}", tag_id);
        Ok(false)
    }
}

#[tauri::command]
pub async fn get_tag_stats(
    pool: tauri::State<'_, sqlx::PgPool>
) -> Result<serde_json::Value, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or("User not logged in".to_string())?;
    
    println!("📊 Getting tag stats for organization: {}", organization_id);
    
    let tag_repo = TagRepository::new(pool.inner().clone());
    let stats: Vec<crate::db::schemas::tags::TagStats> = tag_repo.get_tag_stats(&organization_id) // Specify type
        .await
        .map_err(|e: sqlx::Error| format!("Failed to get tag stats: {}", e))?;
    
    let tags: Vec<Tag> = tag_repo.get_organization_tags(&organization_id) // Specify type
        .await
        .map_err(|e: sqlx::Error| format!("Failed to get tags: {}", e))?;
    
    let response = serde_json::json!({
        "total_tags": tags.len(),
        "tag_usage": stats,
        "organization_id": organization_id
    });
    
    println!("✅ Found {} tags with usage statistics", tags.len());
    
    Ok(response)
}

#[tauri::command]
pub async fn assign_tag_to_entry(
    app_handle: tauri::AppHandle,
    clipboard_entry_id: i64,
    tag_name: String, // Change from tag_id to tag_name
) -> Result<serde_json::Value, String> {
    let pool = app_handle.try_state::<PgPool>()
        .ok_or("Database pool not available")?;
    
    match crate::db::database::ClipboardRepository::assign_tag(pool.inner(), clipboard_entry_id, &tag_name).await {
        Ok(entry) => Ok(serde_json::to_value(entry).unwrap()),
        Err(e) => Err(format!("Failed to assign tag: {}", e)),
    }
}

#[tauri::command]
pub async fn remove_tag_from_entry(
    app_handle: tauri::AppHandle,
    clipboard_entry_id: i64,
    tag_name: String, // Change from tag_id to tag_name
) -> Result<serde_json::Value, String> {
    let pool = app_handle.try_state::<PgPool>()
        .ok_or("Database pool not available")?;
    
    match crate::db::database::ClipboardRepository::remove_tag(pool.inner(), clipboard_entry_id, &tag_name).await {
        Ok(entry) => Ok(serde_json::to_value(entry).unwrap()),
        Err(e) => Err(format!("Failed to remove tag: {}", e)),
    }
}

//Purge Cadance

#[tauri::command]
pub async fn purge_unpinned_entries(
    pool: tauri::State<'_, sqlx::PgPool>
) -> Result<usize, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or("User not logged in".to_string())?;
    
    crate::db::database::ClipboardRepository::delete_unpinned_entries(&pool, &organization_id)
        .await
        .map_err(|e| e.to_string())
}


#[tauri::command]
pub async fn purge_entries_older_than(
    days: i32,
    pool: tauri::State<'_, sqlx::PgPool>
) -> Result<usize, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or("User not logged in".to_string())?;
    
    crate::db::ClipboardRepository::delete_entries_older_than(&pool, &organization_id, days)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn purge_unpinned_older_than(
    days: i32,
    pool: tauri::State<'_, sqlx::PgPool>
) -> Result<usize, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or("User not logged in".to_string())?;
    
    crate::db::ClipboardRepository::delete_unpinned_older_than(&pool, &organization_id, days)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_purge_cadence_options() -> Result<Vec<&'static str>, String> {
    println!("📋 Getting purge cadence options");
    let options = PurgeCadence::all_options();
    println!("✅ Available options: {:?}", options);
    Ok(options)
}

#[tauri::command]
pub async fn get_current_purge_settings(
    pool: tauri::State<'_, sqlx::PgPool>
) -> Result<serde_json::Value, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or("User not logged in".to_string())?;
    
    println!("📊 Getting current purge settings for organization: {}", organization_id);
    
    // Get user by organization ID (assuming one user per organization or you have a way to get user from org)
    let firebase_uid = crate::session::get_current_user_id()
        .ok_or("User not logged in".to_string())?;
    
    let user = UsersRepository::get_by_firebase_uid(&pool, &firebase_uid)
        .await
        .map_err(|e| format!("Failed to get user: {}", e))?
        .ok_or("User not found".to_string())?;
    
    // Determine if auto purge is enabled (not "Never")
    let auto_purge_enabled = user.purge_cadence != PurgeCadence::Never;
    let current_cadence = user.purge_cadence.to_display_string();
    
    println!("✅ Current settings - Auto Purge: {}, Cadence: {}", auto_purge_enabled, current_cadence);
    
    Ok(serde_json::json!({
        "auto_purge_enabled": auto_purge_enabled,
        "purge_cadence": current_cadence,
        "organization_id": organization_id,
        "available_options": PurgeCadence::all_options()
    }))
}

#[tauri::command]
pub async fn update_purge_cadence(
    purge_cadence: String,
    pool: tauri::State<'_, sqlx::PgPool>
) -> Result<UserResponse, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or("User not logged in".to_string())?;
    
    println!("🔄 Updating purge cadence for organization {} to: {}", organization_id, purge_cadence);
    
    // Get user by Firebase UID (since we don't have direct user ID)
    let firebase_uid = crate::session::get_current_user_id()
        .ok_or("User not logged in".to_string())?;
    
    let user = UsersRepository::get_by_firebase_uid(&pool, &firebase_uid)
        .await
        .map_err(|e| format!("Failed to get user: {}", e))?
        .ok_or("User not found".to_string())?;
    
    // Convert string to PurgeCadence enum
    let cadence = PurgeCadence::from_display_string(&purge_cadence)
        .map_err(|e| format!("Invalid purge cadence: {}", e))?;
    
    // Update using the user's database ID
    let updated_user = UsersRepository::update_purge_cadence(&pool, user.id, cadence)
        .await
        .map_err(|e| format!("Failed to update purge cadence: {}", e))?;
    
    println!("✅ Purge cadence updated successfully to: {}", updated_user.purge_cadence.to_display_string());
    
    Ok(UserResponse::from(updated_user))
}

#[tauri::command]
pub async fn update_auto_purge_settings(
    auto_purge_unpinned: bool,
    purge_cadence: String,
    pool: tauri::State<'_, sqlx::PgPool>
) -> Result<UserResponse, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or("User not logged in".to_string())?;
    
    println!("🔄 Updating auto purge settings for organization {} - Auto Purge: {}, Cadence: {}", 
             organization_id, auto_purge_unpinned, purge_cadence);
    
    // Get user by Firebase UID
    let firebase_uid = crate::session::get_current_user_id()
        .ok_or("User not logged in".to_string())?;
    
    let user = UsersRepository::get_by_firebase_uid(&pool, &firebase_uid)
        .await
        .map_err(|e| format!("Failed to get user: {}", e))?
        .ok_or("User not found".to_string())?;
    
    // Convert string to PurgeCadence enum
    let cadence = PurgeCadence::from_display_string(&purge_cadence)
        .map_err(|e| format!("Invalid purge cadence: {}", e))?;
    
    // Update using the user's database ID
    let updated_user = UsersRepository::update_purge_settings(&pool, user.id, auto_purge_unpinned, cadence)
        .await
        .map_err(|e| format!("Failed to update purge settings: {}", e))?;
    
    println!("✅ Auto purge settings updated successfully for organization: {}", organization_id);
    
    Ok(UserResponse::from(updated_user))
}

//Updater Commands

pub fn setup_silent_auto_updater(app: &tauri::AppHandle) {
    let app_handle = app.clone();
    
    // Check for updates 60 seconds after app starts
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(60)).await;
        
        if let Err(e) = check_and_install_update_silently(&app_handle).await {
            eprintln!("Silent auto-update failed: {}", e);
        }
    });


    
    let app_handle_periodic = app.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(24 * 60 * 60)).await;
            
            if let Err(e) = check_and_install_update_silently(&app_handle_periodic).await {
                eprintln!("Periodic silent update failed: {}", e);
            }
        }
    });
}

pub async fn check_and_install_update_silently(app: &tauri::AppHandle) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    
    match updater.check().await {
        Ok(Some(update)) => {
            let current_version = env!("CARGO_PKG_VERSION");
            println!("🔄 Silent update available: {} → {}", current_version, update.version);
            
            // Download and install silently
            let on_chunk = |_chunk_length: usize, _content_length: Option<u64>| {
                // Silent progress - no UI updates
            };
            
            let on_download_finish = || {
                println!("✅ Update downloaded silently");
            };
            
            let _downloaded_bytes = update.download(on_chunk, on_download_finish).await.map_err(|e| e.to_string())?;
            
            println!("✅ Update installed silently. App will restart on next launch.");
            Ok(())
        }
        Ok(None) => {
            println!("✅ Application is up to date (silent check)");
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Silent update check failed: {}", e);
            Ok(()) // Don't fail, just log and continue
        }
    }
}





#[tauri::command]
pub async fn check_for_updates(_app_handle: AppHandle) -> Result<UpdateCheckResult, String> {
    // Replace with your actual GitHub info
    let updater = Updater::new("Shivanshudeveloper", "clipboard_updates", "0.2.4");
    let result = updater.check_for_updates().await;
    Ok(result)
}

#[tauri::command]
pub async fn install_update(app_handle: AppHandle, download_url: String) -> Result<(), String> {
    let updater = Updater::new("Shivanshudeveloper", "clipboard_updates", "0.2.4");
    updater.download_and_install(download_url, app_handle).await
}

// New commands for in-app downloading:
#[tauri::command]
pub async fn download_update(
    app_handle: AppHandle,
    download_url: String,
    updater_state: State<'_, Mutex<Option<Updater>>>,
) -> Result<InstallerInfo, String> {
    let mut updater_guard = updater_state.lock().await; // Use .await instead of .unwrap()
    
    if updater_guard.is_none() {
        *updater_guard = Some(Updater::new("Shivanshudeveloper", "clipboard_updates", "0.2.4"));
    }
    
    if let Some(updater) = updater_guard.as_mut() {
        updater.download_update(download_url, app_handle).await
    } else {
        Err("Updater not initialized".to_string())
    }
}

#[tauri::command]
pub async fn install_downloaded_update(
    installer_info: InstallerInfo,
    updater_state: State<'_, Mutex<Option<Updater>>>,
) -> Result<(), String> {
    let updater_guard = updater_state.lock().await; // Use .await instead of .unwrap()
    
    if let Some(updater) = updater_guard.as_ref() {
        updater.install_downloaded_update(installer_info).await
    } else {
        Err("Updater not initialized".to_string())
    }
}

#[tauri::command]
pub async fn cancel_update(
    updater_state: State<'_, Mutex<Option<Updater>>>,
) -> Result<(), String> {
    let mut updater_guard = updater_state.lock().await; // Use .await instead of .unwrap()
    
    if let Some(updater) = updater_guard.as_mut() {
        updater.cleanup();
    }
    *updater_guard = None;
    
    Ok(())
}



#[tauri::command]
pub async fn auto_update(app_handle: AppHandle) -> Result<bool, String> {
    let mut updater = Updater::new("Shivanshudeveloper", "clipboard_updates", "0.2.4");
    updater.auto_update(app_handle).await
}


#[tauri::command]
pub async fn check_and_notify_updates(app_handle: AppHandle) -> Result<(), String> {
    let updater = Updater::new("Shivanshudeveloper", "clipboard_updates", "0.2.4");
    updater.check_and_notify(app_handle).await;
    Ok(())
}