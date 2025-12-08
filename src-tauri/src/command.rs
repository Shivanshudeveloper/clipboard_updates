use tauri::command;
use crate::db::schemas::{ClipboardEntry, NewClipboardEntry};
use crate::auth::verify_firebase_token;
use crate::db::users_repository::UsersRepository;
use crate::db::schemas::users::{NewUser, UserResponse, PurgeCadence};
use crate::db::schemas::tags::{Tag, NewTag, UpdateTag, TagResponse};
use crate::db::tags_repository::TagRepository;
use rand::Rng;
use serde_json;
use tauri::{State, Window, Manager};
use sqlx::PgPool;
use tauri_plugin_updater::UpdaterExt;
use std::time::Duration;
use tauri::AppHandle;
use tauri::async_runtime::{self, Mutex};
use crate::updater::{Updater, UpdateCheckResult, InstallerInfo};
use crate::db::database::ClipboardRepository;
use crate::db::sqlite_tags_repository::{SqliteTagRepository, LocalTag};
use sqlx::Row;
use crate::config::{get_github_owner, get_github_repo, get_current_version};
use tauri_plugin_opener::OpenerExt;
use crate::db::sqlite_database::SqliteClipboardRepository;
use crate::google_oauth::{GoogleOAuth, GoogleOAuthConfig, GoogleUserInfo};
use tiny_http::{Server, Response, ListenAddr};
use url::Url;
use uuid::Uuid;
use crate::DbPools;
use crate::db::sqlite_users_repository::SqliteUsersRepository;
use sqlx::SqlitePool;

// ======================= GOOGLE LOGIN =======================

#[tauri::command]
pub async fn google_login(
    window: Window,
    db_pools: State<'_, DbPools>,
) -> Result<UserResponse, String> {
    println!("🔐 Starting Google OAuth login...");

    // Extract Postgres (required for Google login)
    let pg_pool = db_pools
        .pg
        .as_ref()
        .ok_or_else(|| "Cloud database (Postgres) not available".to_string())?;

    // SQLite (always available)
    let sqlite_pool = &db_pools.sqlite;

    // 1) Start loopback OAuth server
    let server = Server::http("127.0.0.1:0")
        .map_err(|e| format!("Failed to start OAuth listener: {}", e))?;

    let port = match server.server_addr() {
        ListenAddr::IP(addr) => addr.port(),
        _ => return Err("Unexpected local server address".into()),
    };

    let redirect_uri = format!("http://127.0.0.1:{}/callback", port);
    println!("🌐 OAuth Redirect URI: {}", redirect_uri);

    // 2) Google OAuth config
    let mut cfg = GoogleOAuthConfig::default();
    cfg.redirect_uri = redirect_uri.clone();
    let oauth = GoogleOAuth::new(cfg);

    // 3) Generate auth URL
    let (auth_url, _state) = oauth.generate_auth_url()?;
    window
        .opener()
        .open_url(auth_url.as_str(), None::<&str>)
        .map_err(|e| format!("Failed to open system browser: {}", e))?;
    println!("🚀 Browser opened for Google login");

    // 4) Wait for callback
    let request = server
        .incoming_requests()
        .next()
        .ok_or_else(|| "No OAuth callback received".to_string())?;

    let full_url = format!("{}{}", redirect_uri, request.url());
    let parsed = Url::parse(&full_url)
        .map_err(|e| format!("Invalid OAuth callback URL: {}", e))?;

    let code = parsed
        .query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| "Missing ?code in callback".to_string())?;

    let state = parsed
        .query_pairs()
        .find(|(k, _)| k == "state")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| "Missing ?state in callback".to_string())?;

    let _ = request.respond(Response::from_string(
        "Login successful! You can close this tab.",
    ));

    // 5) Exchange code for token
    let auth = oauth
        .exchange_code_for_token(&code, &state)
        .await
        .map_err(|e| format!("Failed to exchange code for tokens: {}", e))?;

    // 6) Fetch Google profile
    let user_info = oauth
        .get_user_info(&auth.access_token)
        .await
        .map_err(|e| format!("Failed to fetch Google user info: {}", e))?;

    println!(
        "✅ Google user logged in: {} ({})",
        user_info.email, user_info.sub
    );

    let google_uid = format!("google:{}", user_info.sub);
    let google_email = user_info.email.clone();
    let google_name = user_info.name.clone();

    // ======================================================
    // EXISTING USER CHECK
    // ======================================================
    if let Some(existing_user) =
        UsersRepository::get_by_firebase_uid(pg_pool, &google_uid)
            .await
            .map_err(|e| format!("DB error: {}", e))?
    {
        println!("🟢 Existing Google user found: {}", existing_user.email);

        let org_id = existing_user
            .organization_id
            .clone()
            .unwrap_or_else(|| format!("org_{}", google_uid));

        // Ensure user exists locally
        if SqliteUsersRepository::get_by_firebase_uid(sqlite_pool, &google_uid)
            .await
            .unwrap_or(None)
            .is_none()
        {
            let new_local_user = NewUser {
                firebase_uid: existing_user.firebase_uid.clone(),
                email: existing_user.email.clone(),
                display_name: existing_user.display_name.clone(),
                organization_id: existing_user.organization_id.clone(),
            };

            let _ = SqliteUsersRepository::create_user(sqlite_pool, &new_local_user).await;
            println!("📝 Created local SQLite mirror for user");
        }

        crate::session::set_current_user(
            existing_user.firebase_uid.clone(),
            org_id.clone(),
            existing_user.email.clone(),
        );

        // Bootstrap cloud → local
        // let _ = bootstrap_from_cloud_for_org(pg_pool, sqlite_pool, &org_id).await;

        return Ok(UserResponse::from(existing_user));
    }

    // ======================================================
    // NEW GOOGLE USER
    // ======================================================
    let new_org_id = Uuid::new_v4().to_string();

    let new_user = NewUser {
        firebase_uid: google_uid.clone(),
        email: google_email.clone(),
        display_name: Some(google_name.clone()),
        organization_id: Some(new_org_id.clone()),
    };

    let created = UsersRepository::create_user(pg_pool, &new_user)
        .await
        .map_err(|e| format!("Failed to create Google user in Postgres: {}", e))?;

    // Create local mirror
    let _ = SqliteUsersRepository::create_user(sqlite_pool, &new_user).await;

    crate::session::set_current_user(
        created.firebase_uid.clone(),
        new_org_id.clone(),
        created.email.clone(),
    );

    println!("🎉 New Google user created & session initialized");

    // Import cloud history (likely 0 for new user)
    let _ = bootstrap_from_cloud_for_org(pg_pool, sqlite_pool, &new_org_id).await;

    Ok(UserResponse::from(created))
}

// ======================= CLIPBOARD ENTRIES =======================

#[tauri::command]
pub async fn get_my_entries(
    limit: Option<i64>,
    db_pools: tauri::State<'_, DbPools>,
) -> Result<Vec<ClipboardEntry>, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    let entries: Vec<ClipboardEntry> = SqliteClipboardRepository::get_by_organization(
        &db_pools.sqlite,
        &organization_id,
        limit,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(entries)
}

#[command]
pub async fn get_recent_entries(
    hours: Option<i32>,
    db_pools: tauri::State<'_, DbPools>,
) -> Result<Vec<ClipboardEntry>, String> {
    let hours = hours.unwrap_or(24);

    let pg_pool = db_pools
        .pg
        .as_ref()
        .ok_or_else(|| "Cloud database (Postgres) not available".to_string())?;

    ClipboardRepository::get_recent(pg_pool, hours)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_entry_by_id(
    id: i64,
    db_pools: tauri::State<'_, DbPools>,
) -> Result<Option<ClipboardEntry>, String> {
    let pg_pool = db_pools
        .pg
        .as_ref()
        .ok_or_else(|| "Cloud database (Postgres) not available".to_string())?;

    ClipboardRepository::get_by_id(pg_pool, id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_entry(
    id: i64,
    db_pools: tauri::State<'_, DbPools>,
) -> Result<bool, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    let sqlite_pool = &db_pools.sqlite;

    // 1) Get server_id from SQLite for this local id
    let row = sqlx::query(
        r#"
        SELECT server_id
        FROM clipboard_entries
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(sqlite_pool)
    .await
    .map_err(|e| format!("Failed to fetch server_id from SQLite: {}", e))?;

    let server_id_opt: Option<String> = row
        .as_ref()
        .and_then(|r| r.try_get::<String, _>("server_id").ok());

    // 2) Delete from SQLite (local)
    let deleted_local = SqliteClipboardRepository::delete_entry(sqlite_pool, id)
        .await
        .map_err(|e| e.to_string())?;

    if !deleted_local {
        return Err("Local delete failed".into());
    }

    // 3) Delete from Postgres (cloud) if pool + server_id available
    if let (Some(pg_pool), Some(server_id_str)) = (&db_pools.pg, server_id_opt) {
        if let Ok(server_id) = server_id_str.parse::<i64>() {
            let deleted_cloud =
                ClipboardRepository::delete_entry_for_org(pg_pool, server_id, &organization_id)
                    .await
                    .map_err(|e| e.to_string())?;

            println!("☁️ Cloud delete status: {}", deleted_cloud);
        } else {
            eprintln!("⚠️ Invalid server_id stored in SQLite: '{}'", server_id_str);
        }
    } else {
        println!("ℹ️ No pg pool or no server_id; skipping cloud delete");
    }

    Ok(true)
}

#[command]
pub async fn update_entry_content(
    id: i64,
    new_content: String,
    db_pools: tauri::State<'_, DbPools>,
) -> Result<ClipboardEntry, String> {
    SqliteClipboardRepository::update_entry_content(&db_pools.sqlite, id, &new_content)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn search_entries(
    query: String,
    db_pools: tauri::State<'_, DbPools>,
) -> Result<Vec<ClipboardEntry>, String> {
    SqliteClipboardRepository::search_content(&db_pools.sqlite, &query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_entry(
    id: i64,
    updates: serde_json::Value,
    db_pools: tauri::State<'_, DbPools>, // <- use DbPools
) -> Result<ClipboardEntry, String> {
    use crate::db::schemas::UpdateClipboardEntry;

    let update_struct = UpdateClipboardEntry {
        is_pinned: updates.get("is_pinned").and_then(|v| v.as_bool()),
        tags: updates.get("tags").and_then(|v| {
            if v.is_string() {
                Some(v.as_str().unwrap().to_string())
            } else if v.is_array() {
                serde_json::to_string(v).ok()
            } else {
                None
            }
        }),
        ..Default::default()
    };

    // 🔁 Update in SQLite, mark sync_status='local' inside this fn
    SqliteClipboardRepository::update_entry(&db_pools.sqlite, id, update_struct)
        .await
        .map_err(|e| e.to_string())
}

// ======================= AUTH: LOGIN / SIGNUP / SESSION =======================

#[command]
pub async fn login_user(
    firebase_token: String,
    display_name: String,
    db_pools: State<'_, DbPools>, // Postgres + SQLite from here
) -> Result<UserResponse, String> {
    println!("🔐 Starting login_user command...");
    println!("👤 Received display name: {}", display_name);

    let (uid, email, _) = verify_firebase_token(&firebase_token).await?;
    println!("✅ Firebase UID verified: {}", uid);
    println!("📧 User email: {}", email);

    let pg_pool = db_pools
        .pg
        .as_ref()
        .ok_or_else(|| "Cloud database (Postgres) not available".to_string())?;
    let sqlite_pool: &SqlitePool = &db_pools.sqlite;

    // ✅ Existing user path
    if let Some(user) = UsersRepository::get_by_firebase_uid(pg_pool, &uid)
        .await
        .map_err(|e| e.to_string())?
    {
        println!("🟢 Returning existing user: {}", user.email);

        let real_organization_id = user.organization_id.clone().unwrap_or(uid.clone());
        println!("🏢 Real organization ID from DB: {:?}", real_organization_id);

        // 🔹 Ensure user exists locally in SQLite
        match SqliteUsersRepository::get_by_firebase_uid(sqlite_pool, &uid).await {
            Ok(Some(_local)) => {
                println!("✅ [SQLite] Existing local user found");
            }
            Ok(None) => {
                println!("📝 [SQLite] Creating local user mirror for existing Firebase user");
                let new_local_user = NewUser {
                    firebase_uid: user.firebase_uid.clone(),
                    email: user.email.clone(),
                    display_name: user.display_name.clone(),
                    organization_id: user.organization_id.clone(),
                };
                if let Err(e) =
                    SqliteUsersRepository::create_user(sqlite_pool, &new_local_user).await
                {
                    eprintln!("⚠️ [SQLite] Failed to create local user mirror: {}", e);
                }
            }
            Err(e) => {
                eprintln!("⚠️ [SQLite] Failed to fetch local user: {}", e);
            }
        }

        // Set session
        crate::session::set_current_user(
            user.firebase_uid.clone(),
            real_organization_id.clone(),
            user.email.clone(),
        );
        println!("👤 Session set for existing user");

        // 🌐 Bootstrap cloud → local
        // match bootstrap_from_cloud_for_org(pg_pool, sqlite_pool, &real_organization_id).await {
        //     Ok(count) => println!("✅ Bootstrapped {} entries from cloud", count),
        //     Err(e) => eprintln!("⚠️ Failed to bootstrap clipboard from cloud: {}", e),
        // }

        return Ok(UserResponse::from(user));
    }

    // ✅ New user path
    let new_organization_id = format!("org_{}", uid);

    let new_user = NewUser {
        firebase_uid: uid.clone(),
        email: email.clone(),
        display_name: Some(display_name),
        organization_id: Some(new_organization_id.clone()),
    };

    println!(
        "📝 Creating user with generated organization ID: {}",
        new_organization_id
    );

    match UsersRepository::create_user(pg_pool, &new_user).await {
        Ok(created) => {
            println!("✅ User created successfully");

            // 🔹 Also create local mirror in SQLite
            if let Err(e) = SqliteUsersRepository::create_user(sqlite_pool, &new_user).await {
                eprintln!("⚠️ [SQLite] Failed to create local user mirror: {}", e);
            }

            crate::session::set_current_user(
                created.firebase_uid.clone(),
                new_organization_id.clone(),
                created.email.clone(),
            );
            println!("👤 Session set for new user");

            // For brand new user, cloud likely empty -> imports 0 rows, which is fine
            match bootstrap_from_cloud_for_org(pg_pool, sqlite_pool, &new_organization_id).await {
                Ok(count) => println!("✅ Bootstrapped {} entries from cloud", count),
                Err(e) => eprintln!("⚠️ Failed to bootstrap clipboard from cloud: {}", e),
            }

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
    db_pools: State<'_, DbPools>,
) -> Result<UserResponse, String> {
    println!("🔐 Starting signup_user command...");
    println!("👤 Received display name: {}", display_name);
    println!("🏢 Received organization ID: {}", organization_id);

    let (uid, email, _) = verify_firebase_token(&firebase_token).await?;
    println!("✅ Firebase UID verified: {}", uid);
    println!("📧 User email: {}", email);

    let pg_pool = db_pools
        .pg
        .as_ref()
        .ok_or_else(|| "Cloud database (Postgres) not available".to_string())?;
    let sqlite_pool: &SqlitePool = &db_pools.sqlite;

    if let Some(existing_user) = UsersRepository::get_by_firebase_uid(pg_pool, &uid)
        .await
        .map_err(|e| e.to_string())?
    {
        println!("❌ User already exists: {}", existing_user.email);
        return Err("User already exists. Please login instead.".to_string());
    }

    let new_user = NewUser {
        firebase_uid: uid.clone(),
        email: email.clone(),
        display_name: Some(display_name),
        organization_id: Some(organization_id.clone()),
    };

    println!(
        "📝 Creating user with data - Email: {}, Display Name: {:?}",
        new_user.email, new_user.display_name
    );

    match UsersRepository::create_user(pg_pool, &new_user).await {
        Ok(created) => {
            println!(
                "✅ User created successfully - ID: {}, Email: {}, Name: {:?}",
                created.id, created.email, created.display_name
            );

            // 🔹 Create local mirror in SQLite
            if let Err(e) = SqliteUsersRepository::create_user(sqlite_pool, &new_user).await {
                eprintln!("⚠️ [SQLite] Failed to create local user mirror: {}", e);
            }

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
    println!("Logging out user....");
    Ok("User logged out".to_string())
}

#[tauri::command]
pub async fn validate_session(
    app: AppHandle,
) -> Result<Option<UserResponse>, String> {
    // Get DbPools (SQLite + optional Postgres)
    let db_pools_state = match app.try_state::<DbPools>() {
        Some(p) => p,
        None => {
            println!("⚠️ validate_session: DbPools not ready or not managed yet");
            return Ok(None);
        }
    };

    let sqlite_pool = &db_pools_state.sqlite;
    let pg_pool_opt = db_pools_state.pg.as_ref();

    // In-memory session (this still has to exist)
    let user_id = match crate::session::get_current_user_id() {
        Some(id) => id,
        None => {
            println!("ℹ️ validate_session: No user ID in session");
            return Ok(None);
        }
    };

    println!("🔍 Validating session for user_id={}", user_id);

    // 1) Try LOCAL (SQLite) first
    match SqliteUsersRepository::get_by_firebase_uid(sqlite_pool, &user_id).await {
        Ok(Some(local_user)) => {
            println!("✅ [SQLite] Session valid for user: {}", local_user.email);
            return Ok(Some(UserResponse::from(local_user)));
        }
        Ok(None) => {
            println!("ℹ️ [SQLite] No local user found for firebase_uid={}", user_id);
        }
        Err(e) => {
            eprintln!(
                "⚠️ [SQLite] Error fetching local user in validate_session: {}",
                e
            );
        }
    }

    // 2) Fallback to Postgres if available (for when you're online)
    if let Some(pg_pool) = pg_pool_opt {
        match UsersRepository::get_by_firebase_uid(pg_pool, &user_id).await {
            Ok(Some(user)) => {
                println!("✅ [Postgres] Session valid for user: {}", user.email);

                // Ensure local mirror exists (nice-to-have)
                if let Err(e) = SqliteUsersRepository::create_user(
                    sqlite_pool,
                    &NewUser {
                        firebase_uid: user.firebase_uid.clone(),
                        email: user.email.clone(),
                        display_name: user.display_name.clone(),
                        organization_id: user.organization_id.clone(),
                    },
                )
                .await
                {
                    eprintln!(
                        "⚠️ [SQLite] Failed to create local user mirror in validate_session: {}",
                        e
                    );
                }

                return Ok(Some(UserResponse::from(user)));
            }
            Ok(None) => {
                println!("❌ User not found in Postgres for firebase_uid={}", user_id);
                crate::session::clear_current_user();
                return Ok(None);
            }
            Err(e) => {
                println!("❌ Error validating session via Postgres: {}", e);
                return Err(format!("Failed to validate session: {}", e));
            }
        }
    }

    println!("ℹ️ No DB user found (local or cloud). Clearing session.");
    crate::session::clear_current_user();
    Ok(None)
}

#[tauri::command]
pub async fn restore_session(
    app: AppHandle,
    organization_id: String,
) -> Result<Option<UserResponse>, String> {
    // Grab DbPools from app state
    let db_pools_state = match app.try_state::<DbPools>() {
        Some(p) => p,
        None => {
            println!("⚠️ restore_session: DbPools not ready or not managed yet");
            return Ok(None);
        }
    };

    let sqlite_pool = &db_pools_state.sqlite;
    let pg_pool_opt = db_pools_state.pg.as_ref();

    println!("🔄 Restoring session for organization_id: {}", organization_id);

    // 1) Try LOCAL user via SQLite
    match SqliteUsersRepository::get_by_organization_id(sqlite_pool, &organization_id).await {
        Ok(Some(user)) => {
            let org_id = user
                .organization_id
                .clone()
                .unwrap_or_else(|| organization_id.clone());

            crate::session::set_current_user(
                user.firebase_uid.clone(),
                org_id.clone(),
                user.email.clone(),
            );

            println!(
                "✅ [SQLite] Session restored for user: {} (org: {})",
                user.email, org_id
            );

            return Ok(Some(UserResponse::from(user)));
        }
        Ok(None) => {
            println!(
                "ℹ️ [SQLite] No local user found for organization_id={}, trying Postgres...",
                organization_id
            );
        }
        Err(e) => {
            eprintln!(
                "⚠️ [SQLite] Error fetching user by organization_id in restore_session: {}",
                e
            );
        }
    }

    // 2) Fallback: Postgres (only when online)
    if let Some(pg_pool) = pg_pool_opt {
        match UsersRepository::get_by_organization_id(pg_pool, &organization_id).await {
            Ok(Some(user)) => {
                let org_id = user
                    .organization_id
                    .clone()
                    .unwrap_or_else(|| organization_id.clone());

                crate::session::set_current_user(
                    user.firebase_uid.clone(),
                    org_id.clone(),
                    user.email.clone(),
                );

                println!(
                    "✅ [Postgres] Session restored for user: {} (org: {})",
                    user.email, org_id
                );

                // Ensure local mirror exists
                if let Err(e) = SqliteUsersRepository::create_user(
                    sqlite_pool,
                    &NewUser {
                        firebase_uid: user.firebase_uid.clone(),
                        email: user.email.clone(),
                        display_name: user.display_name.clone(),
                        organization_id: user.organization_id.clone(),
                    },
                )
                .await
                {
                    eprintln!(
                        "⚠️ [SQLite] Failed to create user mirror in restore_session: {}",
                        e
                    );
                }

                return Ok(Some(UserResponse::from(user)));
            }
            Ok(None) => {
                println!("❌ No user found in Postgres for organization_id={}", organization_id);
                crate::session::clear_current_user();
                Ok(None)
            }
            Err(e) => {
                println!("❌ Error restoring session via Postgres: {}", e);
                Err(format!("Failed to restore session: {}", e))
            }
        }
    } else {
        println!("ℹ️ Postgres not available and no local user for org_id. Clearing session.");
        crate::session::clear_current_user();
        Ok(None)
    }
}

#[tauri::command]
pub async fn get_current_user(
    db_pools: State<'_, DbPools>,
) -> Result<Option<UserResponse>, String> {
    // Read in-memory session
    if let Some(session) = crate::session::get_current_session() {
        println!(
            "🔎 get_current_user: found session for user_id={}, org_id={}",
            session.user_id, session.organization_id
        );

        let firebase_uid = &session.user_id;
        let sqlite_pool = &db_pools.sqlite;
        let pg_pool_opt = db_pools.pg.as_ref();

        // 1) Try SQLite first
        match SqliteUsersRepository::get_by_firebase_uid(sqlite_pool, firebase_uid).await {
            Ok(Some(local_user)) => {
                println!(
                    "✅ [SQLite] get_current_user: returning local user {}",
                    local_user.email
                );
                return Ok(Some(UserResponse::from(local_user)));
            }
            Ok(None) => {
                println!(
                    "ℹ️ [SQLite] get_current_user: no local user for firebase_uid={}",
                    firebase_uid
                );
            }
            Err(e) => {
                eprintln!(
                    "⚠️ [SQLite] get_current_user: error fetching local user: {}",
                    e
                );
            }
        }

        // 2) Fallback to Postgres if available
        if let Some(pg_pool) = pg_pool_opt {
            let db_user = UsersRepository::get_by_firebase_uid(pg_pool, firebase_uid)
                .await
                .map_err(|e| format!("DB error fetching current user: {}", e))?;

            if let Some(u) = db_user {
                println!(
                    "✅ [Postgres] get_current_user: returning DB user {}",
                    u.email
                );
                Ok(Some(UserResponse::from(u)))
            } else {
                println!(
                    "⚠️ get_current_user: no DB user found for firebase_uid={}",
                    firebase_uid
                );
                Ok(None)
            }
        } else {
            println!(
                "ℹ️ get_current_user: Postgres not available, and no local user found"
            );
            Ok(None)
        }
    } else {
        println!("ℹ️ get_current_user: no in-memory session set");
        Ok(None)
    }
}

#[tauri::command]
pub async fn debug_session_state() -> serde_json::Value {
    serde_json::json!({
        "user_id": crate::session::get_current_user_id(),
        "organization_id": crate::session::get_current_organization_id(),
        "email": crate::session::get_current_user_email(),
        "is_logged_in": crate::session::is_user_logged_in()
    })
}

// ======================= TAGS =======================

#[tauri::command]
pub async fn get_tags(
    db_pools: State<'_, DbPools>,
) -> Result<Vec<LocalTag>, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    let repo = SqliteTagRepository::new(db_pools.sqlite.clone());

    let tags = repo
        .get_organization_tags_with_server_id(&organization_id)
        .await
        .map_err(|e| format!("Failed to fetch tags from SQLite: {}", e))?;

    Ok(tags)
}

#[tauri::command]
pub async fn get_organization_tags(
    db_pools: tauri::State<'_, DbPools>,
) -> Result<Vec<TagResponse>, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    println!("🏢 Fetching tags for organization: {}", organization_id);

    let sqlite_tag_repo = SqliteTagRepository::new(db_pools.sqlite.clone());

    let tags: Vec<Tag> = sqlite_tag_repo
        .get_organization_tags(&organization_id)
        .await
        .map_err(|e| format!("[SQLite] Failed to fetch tags: {}", e))?;

    println!(
        "✅ [SQLite] Found {} tags for organization: {}",
        tags.len(),
        organization_id
    );

    Ok(tags.into_iter().map(TagResponse::from).collect())
}

#[tauri::command]
pub async fn create_tag(
    name: String,
    color: Option<String>,
    db_pools: tauri::State<'_, DbPools>,
) -> Result<TagResponse, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or_else(|| "User not logged in".to_string())?;

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

    // --- DB handles ---
    let sqlite_pool = &db_pools.sqlite;
    // let pg_pool_opt = db_pools.pg.as_ref(); // (optional sync)

    // --- 1) LOCAL FIRST (SQLite) ---
    let sqlite_tag_repo = SqliteTagRepository::new(sqlite_pool.clone());

    let exists_local: bool = sqlite_tag_repo
        .tag_name_exists(&organization_id, &name)
        .await
        .map_err(|e| format!("[SQLite] Failed to check tag existence: {}", e))?;

    if exists_local {
        return Err(format!("Tag '{}' already exists in this organization", name));
    }

    let new_tag = NewTag {
        organization_id: organization_id.clone(),
        name: name.trim().to_string(),
        color: formatted_color.clone(),
    };

    let created_local: Tag = sqlite_tag_repo
        .create_tag(&new_tag)
        .await
        .map_err(|e| format!("[SQLite] Failed to create tag: {}", e))?;

    println!(
        "✅ [SQLite] Tag created successfully - ID: {}, Name: {}",
        created_local.id, created_local.name
    );

    // Optional: cloud sync happens via sync_clipboard_to_cloud

    Ok(TagResponse::from(created_local))
}

#[tauri::command]
pub async fn update_tag(
    tag_id: i64,
    updates: serde_json::Value,
    db_pools: tauri::State<'_, DbPools>,
) -> Result<TagResponse, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    println!("🏢 Updating tag {} for organization: {}", tag_id, organization_id);
    println!("📝 Updates: {:?}", updates);

    let pg_pool = db_pools
        .pg
        .as_ref()
        .ok_or_else(|| "Cloud database (Postgres) not available".to_string())?;

    let tag_repo = TagRepository::new(pg_pool.clone());

    // Build update struct
    let mut update_struct = UpdateTag::default();

    if let Some(name_value) = updates.get("name") {
        if let Some(name) = name_value.as_str() {
            if !Tag::is_valid_name(name) {
                return Err("Tag name must be between 1 and 50 characters".to_string());
            }

            // Check if new name conflicts with existing tag
            let exists: bool = tag_repo
                .tag_name_exists(&organization_id, name)
                .await
                .map_err(|e: sqlx::Error| format!("Failed to check tag existence: {}", e))?;

            if exists {
                // But allow if it's the same tag being updated
                let current_tag: Option<Tag> = tag_repo
                    .get_tag(tag_id, &organization_id)
                    .await
                    .map_err(|e: sqlx::Error| format!("Failed to get current tag: {}", e))?;

                if let Some(current_tag) = current_tag {
                    if current_tag.name != name {
                        return Err(format!(
                            "Tag '{}' already exists in this organization",
                            name
                        ));
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

    let updated_tag: Tag = tag_repo
        .update_tag(tag_id, &organization_id, &update_struct)
        .await
        .map_err(|e: sqlx::Error| format!("Failed to update tag: {}", e))?
        .ok_or_else(|| "Tag not found".to_string())?;

    println!(
        "✅ Tag updated successfully - ID: {}, Name: {}",
        updated_tag.id, updated_tag.name
    );

    Ok(TagResponse::from(updated_tag))
}

#[tauri::command]
pub async fn update_retain_tags_setting(
    retain_tags: bool,
    db_pools: tauri::State<'_, DbPools>,
) -> Result<UserResponse, String> {
    // Get current logged-in user from in-memory session
    let firebase_uid = crate::session::get_current_user_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    println!(
        "🔄 Updating retain_tags for user (firebase_uid={}): {}",
        firebase_uid, retain_tags
    );

    let sqlite_pool = &db_pools.sqlite;
    let pg_pool_opt = db_pools.pg.as_ref();

    // 1) LOCAL FIRST: update retain_tags in SQLite
    let local_user = SqliteUsersRepository::get_by_firebase_uid(sqlite_pool, &firebase_uid)
        .await
        .map_err(|e| format!("Failed to get local user: {}", e))?
        .ok_or_else(|| "Local user not found".to_string())?;

    let updated_local = SqliteUsersRepository::update_retain_tags(
        sqlite_pool,
        local_user.id,
        retain_tags,
    )
    .await
    .map_err(|e| format!("Failed to update retain_tags in local DB: {}", e))?;

    println!(
        "✅ [SQLite] retain_tags updated successfully for local user {} -> {}",
        updated_local.email, retain_tags
    );

    // 2) BEST-EFFORT SYNC TO POSTGRES
    if let Some(pg_pool) = pg_pool_opt {
        match UsersRepository::get_by_firebase_uid(pg_pool, &firebase_uid).await {
            Ok(Some(cloud_user)) => {
                if let Err(e) =
                    UsersRepository::update_retain_tags(pg_pool, cloud_user.id, retain_tags).await
                {
                    eprintln!("⚠️ Failed to sync retain_tags to Postgres: {}", e);
                } else {
                    println!(
                        "✅ [Postgres] retain_tags synced for user {} -> {}",
                        cloud_user.email, retain_tags
                    );
                }
            }
            Ok(None) => {
                eprintln!(
                    "⚠️ [Postgres] No cloud user found for firebase_uid={}",
                    firebase_uid
                );
            }
            Err(e) => {
                eprintln!(
                    "⚠️ [Postgres] Failed to fetch cloud user for retain_tags sync: {}",
                    e
                );
            }
        }
    } else {
        eprintln!("ℹ️ Postgres pool not available, retain_tags will be synced later");
    }

    // Return local user state (authoritative for the running app)
    Ok(UserResponse::from(updated_local))
}

#[tauri::command]
pub async fn delete_tag(
    tag_id: i64,
    db_pools: tauri::State<'_, DbPools>,
) -> Result<bool, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    println!("🏢 Deleting tag {} for organization: {}", tag_id, organization_id);

    let sqlite_pool = &db_pools.sqlite;
    let pg_pool_opt = db_pools.pg.as_ref();

    // 1️⃣ Try Postgres first if available
    if let Some(pg_pool) = pg_pool_opt {
        let tag_repo = TagRepository::new(pg_pool.clone());

        let pg_tag: Option<Tag> = tag_repo
            .get_tag(tag_id, &organization_id)
            .await
            .map_err(|e: sqlx::Error| format!("Failed to get tag from Postgres: {}", e))?;

        if let Some(tag) = pg_tag {
            println!(
                "🗑️ Deleting tag from Postgres: {} (ID: {})",
                tag.name, tag.id
            );

            let deleted_pg: bool = tag_repo
                .delete_tag(tag_id, &organization_id)
                .await
                .map_err(|e: sqlx::Error| format!("Failed to delete tag from Postgres: {}", e))?;

            if !deleted_pg {
                println!("❌ Tag not deleted in Postgres - ID: {}", tag_id);
                return Ok(false);
            }

            println!(
                "✅ Tag deleted in Postgres - ID: {}, Name: {}",
                tag_id, tag.name
            );

            // Mirror delete in SQLite
            let delete_sqlite_result = sqlx::query(
                r#"
                DELETE FROM tags
                WHERE organization_id = ?1
                  AND (id = ?2 OR server_id = ?2)
                "#,
            )
            .bind(&organization_id)
            .bind(tag_id)
            .execute(sqlite_pool)
            .await;

            match delete_sqlite_result {
                Ok(result) => {
                    let affected = result.rows_affected();
                    if affected > 0 {
                        println!(
                            "🧹 SQLite: deleted {} local tag row(s) for server_id/id = {}",
                            affected, tag_id
                        );
                    } else {
                        println!(
                            "ℹ️ SQLite: no local tag rows found for server_id/id = {} (might be fine)",
                            tag_id
                        );
                    }
                }
                Err(e) => {
                    println!(
                        "⚠️ Failed to delete tag from SQLite for org {} / tag_id {}: {}",
                        organization_id, tag_id, e
                    );
                }
            }

            return Ok(true);
        }
    }

    // 2️⃣ No Postgres tag or Postgres unavailable → try SQLite-only delete
    println!(
        "ℹ️ Tag not found in Postgres OR Postgres unavailable – trying SQLite-only delete"
    );

    let delete_sqlite_result = sqlx::query(
        r#"
        DELETE FROM tags
        WHERE organization_id = ?1
          AND (id = ?2 OR server_id = ?2)
        "#,
    )
    .bind(&organization_id)
    .bind(tag_id)
    .execute(sqlite_pool)
    .await;

    match delete_sqlite_result {
        Ok(result) => {
            let affected = result.rows_affected();
            if affected > 0 {
                println!(
                    "🧹 SQLite-only: deleted {} local tag row(s) for id/server_id = {}",
                    affected, tag_id
                );
                Ok(true)
            } else {
                println!(
                    "❌ Tag not found in Postgres or SQLite for id/server_id = {}",
                    tag_id
                );
                Ok(false)
            }
        }
        Err(e) => {
            println!(
                "⚠️ SQLite-only delete failed for org {} / tag_id {}: {}",
                organization_id, tag_id, e
            );
            Err(format!("Failed to delete tag from SQLite: {}", e))
        }
    }
}

#[tauri::command]
pub async fn get_tag_stats(
    db_pools: tauri::State<'_, DbPools>,
) -> Result<serde_json::Value, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    println!("📊 Getting tag stats for organization: {}", organization_id);

    let pg_pool = db_pools
        .pg
        .as_ref()
        .ok_or_else(|| "Cloud database (Postgres) not available".to_string())?;

    let tag_repo = TagRepository::new(pg_pool.clone());

    let stats: Vec<crate::db::schemas::tags::TagStats> = tag_repo
        .get_tag_stats(&organization_id)
        .await
        .map_err(|e: sqlx::Error| format!("Failed to get tag stats: {}", e))?;

    let tags: Vec<Tag> = tag_repo
        .get_organization_tags(&organization_id)
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
    clipboard_entry_id: i64,
    tag_name: String,
    db_pools: tauri::State<'_, DbPools>,
) -> Result<ClipboardEntry, String> {
    println!("🟢 Assigning tag '{}' to entry {}", tag_name, clipboard_entry_id);

    SqliteClipboardRepository::assign_tag(&db_pools.sqlite, clipboard_entry_id, &tag_name).await
}

#[tauri::command]
pub async fn remove_tag_from_entry(
    clipboard_entry_id: i64,
    tag_name: String,
    db_pools: tauri::State<'_, DbPools>,
) -> Result<ClipboardEntry, String> {
    println!("🔴 Removing tag '{}' from entry {}", tag_name, clipboard_entry_id);

    SqliteClipboardRepository::remove_tag(&db_pools.sqlite, clipboard_entry_id, &tag_name).await
}

// ======================= PURGE / AUTO PURGE =======================

#[tauri::command]
pub async fn purge_unpinned_entries(
    db_pools: tauri::State<'_, DbPools>,
) -> Result<usize, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    println!(
        "🗑️ Purging all unpinned entries for organization: {}",
        organization_id
    );

    SqliteClipboardRepository::delete_unpinned_entries(&db_pools.sqlite, &organization_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn purge_untagged_entries(
    db_pools: tauri::State<'_, DbPools>,
) -> Result<usize, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    println!(
        "🗑️ Purging all unpinned & untagged entries for organization: {}",
        organization_id
    );

    SqliteClipboardRepository::delete_untagged_entries(&db_pools.sqlite, &organization_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn purge_entries_older_than(
    days: i32,
    db_pools: tauri::State<'_, DbPools>,
) -> Result<usize, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    SqliteClipboardRepository::delete_entries_older_than(
        &db_pools.sqlite,
        &organization_id,
        days,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn purge_unpinned_older_than(
    days: i32,
    db_pools: tauri::State<'_, DbPools>,
) -> Result<usize, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    SqliteClipboardRepository::delete_unpinned_older_than(
        &db_pools.sqlite,
        &organization_id,
        days,
    )
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
    db_pools: tauri::State<'_, DbPools>,
) -> Result<serde_json::Value, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    let firebase_uid = crate::session::get_current_user_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    println!(
        "📊 Getting current purge settings for user={} org={}",
        firebase_uid, organization_id
    );

    let sqlite_pool = &db_pools.sqlite;
    let pg_pool_opt = db_pools.pg.as_ref();

    // 1) Try LOCAL first
    let user = match SqliteUsersRepository::get_by_firebase_uid(sqlite_pool, &firebase_uid).await {
        Ok(Some(u)) => {
            println!("✅ [SQLite] Using local user for purge settings: {}", u.email);
            u
        }
        Ok(None) => {
            println!("ℹ️ [SQLite] No local user found, trying Postgres...");
            if let Some(pg_pool) = pg_pool_opt {
                UsersRepository::get_by_firebase_uid(pg_pool, &firebase_uid)
                    .await
                    .map_err(|e| format!("Failed to get user from Postgres: {}", e))?
                    .ok_or_else(|| "User not found in Postgres".to_string())?
            } else {
                return Err(
                    "User not found locally and cloud database unavailable".to_string()
                );
            }
        }
        Err(e) => {
            eprintln!(
                "⚠️ [SQLite] Error fetching local user: {}. Falling back to Postgres.",
                e
            );
            if let Some(pg_pool) = pg_pool_opt {
                UsersRepository::get_by_firebase_uid(pg_pool, &firebase_uid)
                    .await
                    .map_err(|e| format!("Failed to get user from Postgres: {}", e))?
                    .ok_or_else(|| "User not found in Postgres".to_string())?
            } else {
                return Err(
                    "User not found locally and cloud database unavailable".to_string()
                );
            }
        }
    };

    let auto_purge_enabled = user.purge_cadence != PurgeCadence::Never;
    let current_cadence = user.purge_cadence.to_display_string();

    println!(
        "✅ Current settings - Auto Purge: {}, Cadence: {}",
        auto_purge_enabled, current_cadence
    );

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
    db_pools: tauri::State<'_, DbPools>,
) -> Result<UserResponse, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    let firebase_uid = crate::session::get_current_user_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    println!(
        "🔄 Updating purge cadence for org {} (user={}) to: {}",
        organization_id, firebase_uid, purge_cadence
    );

    let sqlite_pool = &db_pools.sqlite;
    let pg_pool_opt = db_pools.pg.as_ref();

    // Convert string to PurgeCadence enum
    let cadence = PurgeCadence::from_display_string(&purge_cadence)
        .map_err(|e| format!("Invalid purge cadence: {}", e))?;

    // 1) LOCAL FIRST: update in SQLite
    let local_user = SqliteUsersRepository::get_by_firebase_uid(sqlite_pool, &firebase_uid)
        .await
        .map_err(|e| format!("Failed to get local user: {}", e))?
        .ok_or_else(|| "Local user not found".to_string())?;

    let updated_local = SqliteUsersRepository::update_purge_cadence(
        sqlite_pool,
        local_user.id,
        cadence.clone(),
    )
    .await
    .map_err(|e| format!("Failed to update purge cadence in local DB: {}", e))?;

    println!(
        "✅ [SQLite] Purge cadence updated locally to: {}",
        updated_local.purge_cadence.to_display_string()
    );

    // 2) BEST-EFFORT SYNC TO POSTGRES
    if let Some(pg_pool) = pg_pool_opt {
        match UsersRepository::get_by_firebase_uid(pg_pool, &firebase_uid).await {
            Ok(Some(cloud_user)) => {
                if let Err(e) =
                    UsersRepository::update_purge_cadence(pg_pool, cloud_user.id, cadence).await
                {
                    eprintln!("⚠️ Failed to sync purge cadence to Postgres: {}", e);
                } else {
                    println!(
                        "✅ [Postgres] Purge cadence synced for user {}",
                        cloud_user.email
                    );
                }
            }
            Ok(None) => {
                eprintln!(
                    "⚠️ [Postgres] No cloud user found for firebase_uid={} (purge cadence sync skipped)",
                    firebase_uid
                );
            }
            Err(e) => {
                eprintln!(
                    "⚠️ [Postgres] Error fetching cloud user for purge cadence sync: {}",
                    e
                );
            }
        }
    } else {
        eprintln!("ℹ️ Postgres pool not available, purge cadence will be synced later");
    }

    Ok(UserResponse::from(updated_local))
}

#[tauri::command]
pub async fn update_auto_purge_settings(
    auto_purge_unpinned: bool,
    purge_cadence: String,
    db_pools: tauri::State<'_, DbPools>,
) -> Result<UserResponse, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    let firebase_uid = crate::session::get_current_user_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    println!(
        "🔄 Updating auto purge settings for org {} (user={}) - Auto Purge: {}, Cadence: {}",
        organization_id, firebase_uid, auto_purge_unpinned, purge_cadence
    );

    let sqlite_pool = &db_pools.sqlite;
    let pg_pool_opt = db_pools.pg.as_ref();

    // Convert string to PurgeCadence enum
    let cadence = PurgeCadence::from_display_string(&purge_cadence)
        .map_err(|e| format!("Invalid purge cadence: {}", e))?;

    // 1) LOCAL FIRST: update in SQLite
    let local_user = SqliteUsersRepository::get_by_firebase_uid(sqlite_pool, &firebase_uid)
        .await
        .map_err(|e| format!("Failed to get local user: {}", e))?
        .ok_or_else(|| "Local user not found".to_string())?;

    let updated_local = SqliteUsersRepository::update_purge_settings(
        sqlite_pool,
        local_user.id,
        auto_purge_unpinned,
        cadence.clone(),
    )
    .await
    .map_err(|e| format!("Failed to update purge settings in local DB: {}", e))?;

    println!(
        "✅ [SQLite] Auto purge settings updated locally - Enabled: {}, Cadence: {}",
        updated_local.purge_cadence != PurgeCadence::Never,
        updated_local.purge_cadence.to_display_string()
    );

    // 2) BEST-EFFORT SYNC TO POSTGRES
    if let Some(pg_pool) = pg_pool_opt {
        match UsersRepository::get_by_firebase_uid(pg_pool, &firebase_uid).await {
            Ok(Some(cloud_user)) => {
                if let Err(e) = UsersRepository::update_purge_settings(
                    pg_pool,
                    cloud_user.id,
                    auto_purge_unpinned,
                    cadence,
                )
                .await
                {
                    eprintln!("⚠️ Failed to sync auto purge settings to Postgres: {}", e);
                } else {
                    println!(
                        "✅ [Postgres] Auto purge settings synced for user {}",
                        cloud_user.email
                    );
                }
            }
            Ok(None) => {
                eprintln!(
                    "⚠️ [Postgres] No cloud user found for firebase_uid={} (auto purge sync skipped)",
                    firebase_uid
                );
            }
            Err(e) => {
                eprintln!(
                    "⚠️ [Postgres] Error fetching cloud user for auto purge sync: {}",
                    e
                );
            }
        }
    } else {
        eprintln!("ℹ️ Postgres pool not available, auto purge settings will be synced later");
    }

    Ok(UserResponse::from(updated_local))
}

#[tauri::command]
pub async fn run_auto_purge_now(
    db_pools: tauri::State<'_, DbPools>,
) -> Result<usize, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    let firebase_uid = crate::session::get_current_user_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    let sqlite_pool = &db_pools.sqlite;

    let local_user = SqliteUsersRepository::get_by_firebase_uid(sqlite_pool, &firebase_uid)
        .await
        .map_err(|e| format!("Failed to get local user: {}", e))?
        .ok_or_else(|| "Local user not found".to_string())?;

    let Some(days) = local_user.purge_cadence.to_days_i32() else {
        println!("ℹ️ [AUTO PURGE] cadence = Never, skipping");
        return Ok(0);
    };

    println!(
        "🧹 [AUTO PURGE] org={} cadence={:?} → days={} (SQLite)",
        organization_id, local_user.purge_cadence, days
    );

    let deleted = SqliteClipboardRepository::delete_unpinned_older_than(
        sqlite_pool,
        &organization_id,
        days,
    )
    .await
    .map_err(|e| format!("Failed to auto purge unpinned entries: {}", e))?;

    println!(
        "✅ [AUTO PURGE] Deleted {} entries for org {}",
        deleted, organization_id
    );

    Ok(deleted)
}

// ======================= UPDATER =======================

pub fn setup_silent_auto_updater(app: &AppHandle) {
    let app_handle = app.clone();

    // Check for updates 60 seconds after app starts
    async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(60)).await;

        if let Err(e) = check_and_install_update_silently(&app_handle).await {
            eprintln!("Silent auto-update failed: {}", e);
        }
    });

    let app_handle_periodic = app.clone();
    async_runtime::spawn(async move {
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
            println!(
                "🔄 Silent update available: {} → {}",
                current_version, update.version
            );

            let on_chunk = |_chunk_length: usize, _content_length: Option<u64>| {
                // Silent progress - no UI updates
            };

            let on_download_finish = || {
                println!("✅ Update downloaded silently");
            };

            let _downloaded_bytes = update
                .download(on_chunk, on_download_finish)
                .await
                .map_err(|e| e.to_string())?;

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
    let updater = Updater::new(
        get_github_owner(),
        get_github_repo(),
        get_current_version(),
    );
    let result = updater.check_for_updates().await;
    Ok(result)
}

#[tauri::command]
pub async fn install_update(app_handle: AppHandle, download_url: String) -> Result<(), String> {
    let updater = Updater::new(
        get_github_owner(),
        get_github_repo(),
        get_current_version(),
    );
    updater.download_and_install(download_url, app_handle).await
}

#[tauri::command]
pub async fn download_update(
    app_handle: AppHandle,
    download_url: String,
    updater_state: State<'_, Mutex<Option<Updater>>>,
) -> Result<InstallerInfo, String> {
    let mut updater_guard = updater_state.lock().await;
    let github_owner = get_github_owner();
    let github_repo = get_github_repo();
    let current_version = get_current_version();

    if updater_guard.is_none() {
        *updater_guard = Some(Updater::new(github_owner, github_repo, current_version));
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
    let updater_guard = updater_state.lock().await;

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
    let mut updater_guard = updater_state.lock().await;

    if let Some(updater) = updater_guard.as_mut() {
        updater.cleanup();
    }
    *updater_guard = None;

    Ok(())
}

#[tauri::command]
pub async fn auto_update(app_handle: AppHandle) -> Result<bool, String> {
    let mut updater = Updater::new(
        get_github_owner(),
        get_github_repo(),
        get_current_version(),
    );

    updater.auto_update(app_handle).await
}

#[tauri::command]
pub async fn check_and_notify_updates(app_handle: AppHandle) -> Result<(), String> {
    let updater = Updater::new(
        get_github_owner(),
        get_github_repo(),
        get_current_version(),
    );
    updater.check_and_notify(app_handle).await;
    Ok(())
}

#[tauri::command]
pub async fn get_current_user_retain_tags(
    db_pools: tauri::State<'_, DbPools>,
) -> Result<bool, String> {
    // 1. Get current organization
    let organization_id = crate::session::get_current_organization_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    // 2. Access SQLite pool
    let sqlite_pool = &db_pools.sqlite;

    // 3. Fetch local user by organization_id
    let user_opt = SqliteUsersRepository::get_by_organization_id(sqlite_pool, &organization_id)
        .await
        .map_err(|e| format!("Failed to load user from SQLite: {}", e))?;

    // 4. If user exists → return retain_tags
    if let Some(user) = user_opt {
        Ok(user.retain_tags)
    } else {
        Err("User not found in local SQLite".to_string())
    }
}

// ======================= SYNC & BOOTSTRAP (OFFLINE) =======================

#[tauri::command]
pub async fn sync_clipboard_to_cloud(
    db_pools: tauri::State<'_, DbPools>,
) -> Result<usize, String> {
    sync_clipboard_to_cloud_internal(&db_pools).await
}

pub async fn sync_clipboard_to_cloud_internal(
    db_pools: &DbPools,
) -> Result<usize, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    let pg_pool = db_pools
        .pg
        .as_ref()
        .ok_or_else(|| "Cloud database (Postgres) not available".to_string())?;

    let sqlite_pool = &db_pools.sqlite;

    println!("🔄 Starting sync from SQLite → Neon for org: {}", organization_id);

    // ======================================================
    // 1) CLIPBOARD ENTRIES
    // ======================================================
    let pending_entries = SqliteClipboardRepository::get_pending_sync_entries_for_org(
        sqlite_pool,
        &organization_id,
        Some(500), // batch size
    )
    .await
    .map_err(|e| format!("Failed to fetch pending entries from SQLite: {}", e))?;

    if pending_entries.is_empty() {
        println!(
            "ℹ️ No pending clipboard entries to sync for org {}",
            organization_id
        );
    } else {
        println!(
            "📦 Found {} pending clipboard entries to sync",
            pending_entries.len()
        );
    }

    let mut synced_entries = 0usize;

    for local in pending_entries {
        let new_entry = crate::db::schemas::NewClipboardEntry {
            content: local.content.clone(),
            content_type: local.content_type.clone(),
            content_hash: local.content_hash.clone(),
            source_app: local.source_app.clone(),
            source_window: local.source_window.clone(),
            timestamp: local.timestamp,
            tags: local.tags.clone(),
            is_pinned: local.is_pinned,
            organization_id: local.organization_id.clone(),
        };

        let save_result = ClipboardRepository::save_entry(pg_pool, new_entry)
            .await
            .map_err(|e| e.to_string());

        match save_result {
            Ok(cloud_entry) => {
                if let Err(e) = SqliteClipboardRepository::mark_as_synced(
                    sqlite_pool,
                    local.id,
                    cloud_entry.id,
                )
                .await
                {
                    eprintln!(
                        "⚠️ Failed to mark local entry {} as synced: {}",
                        local.id, e
                    );
                } else {
                    synced_entries += 1;
                }
            }
            Err(err_msg) => {
                eprintln!(
                    "❌ Failed to sync local entry {} to Neon: {}",
                    local.id, err_msg
                );
            }
        }
    }

    // ======================================================
    // 2) TAGS
    // ======================================================
    let sqlite_tag_repo = SqliteTagRepository::new(sqlite_pool.clone());
    let pg_tag_repo = TagRepository::new(pg_pool.clone());

    let pending_tags = sqlite_tag_repo
        .get_pending_sync_tags_for_org(&organization_id, Some(500))
        .await
        .map_err(|e| format!("Failed to fetch pending tags from SQLite: {}", e))?;

    if pending_tags.is_empty() {
        println!(
            "ℹ️ No pending tags to sync for org {}",
            organization_id
        );
    } else {
        println!("🏷️ Found {} pending tags to sync", pending_tags.len());
    }

    let mut synced_tags = 0usize;

    for local_tag in pending_tags {
        let new_tag = crate::db::schemas::tags::NewTag {
            organization_id: local_tag.organization_id.clone(),
            name: local_tag.name.clone(),
            color: local_tag.color.clone(),
        };

        let save_result = pg_tag_repo.create_tag(&new_tag).await;

        match save_result {
            Ok(cloud_tag) => {
                if let Err(e) = sqlite_tag_repo
                    .mark_as_synced(local_tag.id, cloud_tag.id)
                    .await
                {
                    eprintln!(
                        "⚠️ Failed to mark local tag {} as synced: {}",
                        local_tag.id, e
                    );
                } else {
                    synced_tags += 1;
                }
            }
            Err(e) => {
                eprintln!(
                    "❌ Failed to sync local tag {} to Postgres: {}",
                    local_tag.id, e
                );
            }
        }
    }

    // ======================================================
    // 3) USER SETTINGS: purge_cadence + retain_tags
    // ======================================================
    let mut synced_user_settings = 0usize;

    let local_user_opt =
        SqliteUsersRepository::get_by_organization_id(sqlite_pool, &organization_id)
            .await
            .map_err(|e| format!("Failed to fetch local user from SQLite: {}", e))?;

    if let Some(local_user) = local_user_opt {
        let cloud_user_opt =
            UsersRepository::get_by_firebase_uid(pg_pool, &local_user.firebase_uid)
                .await
                .map_err(|e| format!("Failed to fetch cloud user from Postgres: {}", e))?;

        if let Some(cloud_user) = cloud_user_opt {
            let mut changed = false;

            // purge_cadence
            if cloud_user.purge_cadence != local_user.purge_cadence {
                println!(
                    "🕒 Syncing purge_cadence for user {}: {:?} → {:?}",
                    cloud_user.id, cloud_user.purge_cadence, local_user.purge_cadence
                );

                UsersRepository::update_purge_cadence(
                    pg_pool,
                    cloud_user.id,
                    local_user.purge_cadence,
                )
                .await
                .map_err(|e| format!("Failed to update cloud purge_cadence: {}", e))?;

                changed = true;
            }

            // retain_tags
            if cloud_user.retain_tags != local_user.retain_tags {
                println!(
                    "🏷️ Syncing retain_tags for user {}: {} → {}",
                    cloud_user.id, cloud_user.retain_tags, local_user.retain_tags
                );

                UsersRepository::update_retain_tags(
                    pg_pool,
                    cloud_user.id,
                    local_user.retain_tags,
                )
                .await
                .map_err(|e| format!("Failed to update cloud retain_tags: {}", e))?;

                changed = true;
            }

            if changed {
                synced_user_settings += 1;
                println!(
                    "✅ Synced user settings for org {} (purge_cadence + retain_tags)",
                    organization_id
                );
            } else {
                println!(
                    "ℹ️ User settings already in sync for org {}",
                    organization_id
                );
            }
        } else {
            eprintln!(
                "ℹ️ No matching cloud user found for firebase_uid {}, skipping user settings sync",
                local_user.firebase_uid
            );
        }
    } else {
        println!(
            "ℹ️ No local user found in SQLite for org {}, skipping user settings sync",
            organization_id
        );
    }

    println!(
        "✅ Sync completed → {} clipboard entries + {} tags + {} user settings",
        synced_entries, synced_tags, synced_user_settings
    );

    Ok(synced_entries + synced_tags + synced_user_settings)
}

#[tauri::command]
pub async fn bootstrap_cloud_now(
    db_pools: tauri::State<'_, DbPools>,
) -> Result<usize, String> {
    let organization_id = crate::session::get_current_organization_id()
        .ok_or_else(|| "User not logged in".to_string())?;

    let pg_pool = db_pools
        .pg
        .as_ref()
        .ok_or_else(|| "Cloud database (Postgres) not available".to_string())?;

    let sqlite_pool = &db_pools.sqlite;

    println!(
        "☁️ [bootstrap_cloud_now] Bootstrapping clipboard from cloud for org: {}",
        organization_id
    );

    bootstrap_from_cloud_for_org(pg_pool, sqlite_pool, &organization_id).await
}

async fn bootstrap_from_cloud_for_org(
    pg_pool: &PgPool,
    sqlite_pool: &SqlitePool,
    organization_id: &str,
) -> Result<usize, String> {
    println!("☁️ Bootstrapping clipboard from cloud for org: {}", organization_id);

    // ======================================================
    // 1) CLIPBOARD ENTRIES (Postgres → SQLite)
    // ======================================================
    let remote_entries: Vec<ClipboardEntry> = ClipboardRepository::get_by_organization(
        pg_pool,
        organization_id,
        None,
    )
    .await
    .map_err(|e| format!("Failed to fetch remote entries from Postgres: {}", e))?;

    println!(
        "☁️ Got {} remote clipboard entries for org {}",
        remote_entries.len(),
        organization_id
    );

    let mut changed_entries = 0usize;

    for remote in remote_entries {
        // 1) Do we already have this Postgres row locally?
        let local_opt = SqliteClipboardRepository::get_by_server_id(sqlite_pool, remote.id)
            .await
            .map_err(|e| format!("Failed to check local by server_id: {}", e))?;

        if let Some(local) = local_opt {
            // 2) UPDATE existing local row with latest remote content
            if let Err(e) =
                SqliteClipboardRepository::update_from_remote(sqlite_pool, local.id, &remote).await
            {
                eprintln!(
                    "❌ Failed to update local entry {} from remote {}: {}",
                    local.id, remote.id, e
                );
                continue;
            }
        } else {
            // 3) INSERT new local row for this remote entry
            if let Err(e) =
                SqliteClipboardRepository::insert_from_remote(sqlite_pool, &remote).await
            {
                eprintln!(
                    "❌ Failed to insert remote entry {} into SQLite: {}",
                    remote.id, e
                );
                continue;
            }
        }

        changed_entries += 1;
    }

    println!(
        "✅ Bootstrapped/updated {} clipboard entries from cloud → local for org {}",
        changed_entries, organization_id
    );

    // ======================================================
    // 2) TAGS (Postgres → SQLite)
    // ======================================================
    println!("☁️ Bootstrapping tags from cloud for org: {}", organization_id);

    let pg_tag_repo = TagRepository::new(pg_pool.clone());
    let sqlite_tag_repo = SqliteTagRepository::new(sqlite_pool.clone());

    let remote_tags = pg_tag_repo
        .get_organization_tags(organization_id)
        .await
        .map_err(|e| format!("Failed to fetch remote tags from Postgres: {}", e))?;

    println!(
        "☁️ Got {} remote tags for org {}",
        remote_tags.len(),
        organization_id
    );

    let mut changed_tags = 0usize;

    for remote_tag in remote_tags {
        // Check if we already have a local tag mapped to this cloud tag
        let local_opt = sqlx::query_as::<_, LocalTag>(
            r#"
            SELECT 
                id,
                organization_id,
                name,
                color,
                created_at,
                updated_at,
                sync_status,
                server_id
            FROM tags
            WHERE organization_id = ?1
              AND server_id = ?2
            LIMIT 1
            "#,
        )
        .bind(organization_id)
        .bind(remote_tag.id)
        .fetch_optional(sqlite_pool)
        .await
        .map_err(|e| format!("Failed to check local tag by server_id: {}", e))?;

        if let Some(local_tag) = local_opt {
            // UPDATE existing local tag with latest name/color/timestamps
            if let Err(e) = sqlx::query(
                r#"
                UPDATE tags
                SET name = ?1,
                    color = ?2,
                    created_at = ?3,
                    updated_at = ?4,
                    sync_status = 'synced'
                WHERE id = ?5
                "#,
            )
            .bind(&remote_tag.name)
            .bind(&remote_tag.color)
            .bind(remote_tag.created_at)
            .bind(remote_tag.updated_at)
            .bind(local_tag.id)
            .execute(sqlite_pool)
            .await
            {
                eprintln!(
                    "❌ Failed to update local tag {} from remote {}: {}",
                    local_tag.id, remote_tag.id, e
                );
                continue;
            }
        } else {
            // INSERT new local tag row mapped to this cloud tag
            if let Err(e) = sqlx::query(
                r#"
                INSERT INTO tags (
                    organization_id,
                    name,
                    color,
                    created_at,
                    updated_at,
                    sync_status,
                    server_id
                )
                VALUES (?1, ?2, ?3, ?4, ?5, 'synced', ?6)
                "#,
            )
            .bind(&remote_tag.organization_id)
            .bind(&remote_tag.name)
            .bind(&remote_tag.color)
            .bind(remote_tag.created_at)
            .bind(remote_tag.updated_at)
            .bind(remote_tag.id)
            .execute(sqlite_pool)
            .await
            {
                eprintln!(
                    "❌ Failed to insert remote tag {} into SQLite: {}",
                    remote_tag.id, e
                );
                continue;
            }
        }

        changed_tags += 1;
    }

    println!(
        "✅ Bootstrapped/updated {} tags from cloud → local for org {}",
        changed_tags, organization_id
    );

    println!(
        "✅ Full bootstrap completed → {} clipboard entries + {} tags for org {}",
        changed_entries, changed_tags, organization_id
    );

    Ok(changed_entries + changed_tags)
}


