#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod commands;
mod command;
mod auth;
mod session;
mod updater;

use tauri::{
    Manager, Emitter,
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent},
    PhysicalPosition, PhysicalSize, State,
};
use tauri::{WebviewWindowBuilder,  WindowEvent};
use tauri_utils::config::WebviewUrl;
use std::time::Duration;

use std::sync::atomic::{AtomicBool, Ordering};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_store::Builder as StoreBuilder;
use winreg::enums::*;
use winreg::RegKey;
use tokio::time;

use command::setup_silent_auto_updater;

use command::{
    // Core clipboard operations
    get_recent_entries,
    get_entry_by_id,
    get_my_entries,
    delete_entry,
    update_entry,
    update_entry_content,
    search_entries,

    // Organization & tagging
    get_organization_tags,
    get_tags,
    create_tag,
    update_tag,
    delete_tag,
    get_tag_stats,
    assign_tag_to_entry,
    remove_tag_from_entry,

    // Data management
    purge_unpinned_entries,
    purge_untagged_entries,
    purge_entries_older_than,
    purge_unpinned_older_than,
    get_purge_cadence_options,
    get_current_purge_settings,
    update_purge_cadence,
    update_auto_purge_settings,
    update_retain_tags_setting,
    get_current_user_retain_tags,
    run_auto_purge_now,

    // User management
    login_user,
    logout_user,
    signup_user,
    validate_session,
    restore_session,
    google_login,
    debug_session_state,
    get_current_user,
    get_user_plan,

    // Application updates
    install_update,
    download_update,
    install_downloaded_update,
    cancel_update,
    auto_update,
    check_and_notify_updates,
    check_for_updates,
    sync_clipboard_to_cloud,
    bootstrap_cloud_now,

    // Payment operations
    open_payment_website,
    check_payment_status,
    refresh_user_plan_from_backend,
};

use crate::updater::Updater;
use crate::commands::clipboard::start_clipboard_monitoring;
mod config;
mod google_oauth;
use sqlx::{PgPool, SqlitePool};

use crate::config::{get_github_owner, get_github_repo, get_current_version};
use crate::db::database::create_db_pool;
use crate::db::sqlite_database::create_sqlite_pool;
use crate::db::sqlite_database::SqliteClipboardRepository;
use crate::session::{set_current_user, get_current_user_id, get_current_organization_id, get_current_user_email, get_current_session, clear_current_user, is_user_logged_in};


// 🔁 Use Tauri's async runtime for spawn + Mutex
use tauri::async_runtime::{self, Mutex};

// Application configuration
const POP_W: f64 = 460.0;
const MIN_POP_H: f64 = 850.0;
const MAX_POP_H: f64 = 900.0;

// Application state
#[derive(Debug)]
pub struct AppState {
    pub is_database_ready: AtomicBool,
    pub is_clipboard_monitoring: AtomicBool,
}

#[derive(Debug)]
pub struct DbPools {
    pub pg: Option<PgPool>,
    pub sqlite: SqlitePool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            is_database_ready: AtomicBool::new(false),
            is_clipboard_monitoring: AtomicBool::new(false),
        }
    }
}

fn main() {
    println!("🚀 Starting ClipTray v{}...", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .plugin(StoreBuilder::default().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None
        ))
        .manage(AppState::default())
        .manage(Mutex::new(Option::<Updater>::None))
        .setup(|app| {
            let app_handle = app.handle().clone();

            enable_auto_start_silent();

            let was_auto_started = std::env::args().any(|arg| arg == "--silent");

            if was_auto_started {
                println!("🚀 Application auto-started on boot");
            } else {
                println!("👤 Application started manually by user");
            }

            println!("✅ Auto-start configured");

            // ✅ Setup tray + UI immediately
            setup_tray_and_ui(app)?;

            if let Err(e) = ensure_main_window(&app_handle) {
                eprintln!("❌ Error opening main window: {}", e);
            }

            if let Err(e) = ensure_pricing_window(&app_handle) {
    eprintln!("❌ Failed to create pricing window: {}", e);
}
           

            // ✅ Background initialization on Tauri's async runtime
            async_runtime::spawn(async move {
                if let Err(e) = initialize_application_async(app_handle.clone()).await {
                    eprintln!("❌ Background initialization failed: {}", e);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_pricing_window,
            // Entry operations
            get_my_entries,
            get_recent_entries,
            get_entry_by_id,
            delete_entry,
            update_entry,
            update_entry_content,
            search_entries,

            // Tag operations
            get_organization_tags,
            get_tags,
            create_tag,
            update_tag,
            delete_tag,
            get_tag_stats,
            assign_tag_to_entry,
            remove_tag_from_entry,

            // Purge operations
            purge_entries_older_than,
            purge_unpinned_entries,
            purge_untagged_entries,
            purge_unpinned_older_than,
            get_purge_cadence_options,
            get_current_purge_settings,
            update_purge_cadence,
            update_auto_purge_settings,
            get_current_user_retain_tags,
            update_retain_tags_setting,
            run_auto_purge_now,

            // User authentication
            login_user,
            logout_user,
            signup_user,
            validate_session,
            google_login,
            debug_session_state,
            get_current_user,
            restore_session,
            get_user_plan,

            // Update operations
            install_update,
            download_update,
            install_downloaded_update,
            cancel_update,
            auto_update,
            check_for_updates,
            check_and_notify_updates,

            // Autostart
            enable_auto_start,

            // Window management
            resize_window,

            commands::editor::open_in_notepad_and_wait,

            // Database status + sync
            check_database_status,
            sync_clipboard_to_cloud,
            bootstrap_cloud_now,

            // Payment operations
            open_payment_website,
            check_payment_status,
            refresh_user_plan_from_backend,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
            if window.label() == "main" {
                if let tauri::WindowEvent::Focused(false) = event {
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("❌ Error while running Tauri application");
}



// === AUTOSTART ===

fn set_autostart_registry() -> Result<(), String> {
    use std::env;

    let exe_path = env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;

    let exe_path_str = exe_path
        .to_str()
        .ok_or_else(|| "Invalid executable path (non-UTF8)".to_string())?;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
        .map_err(|e| format!("Failed to open registry key: {}", e))?;

    let value = format!("\"{}\"", exe_path_str); // Removed --silent

    key.set_value("ClipTray", &value)
        .map_err(|e| format!("Failed to set registry value: {}", e))?;

    Ok(())
}


#[tauri::command]
fn enable_auto_start() -> Result<bool, String> {
    set_autostart_registry()?;
    println!("✅ Auto-start enabled via registry (no cmd window)");
    Ok(true)
}

/// ✅ Enable auto-start silently without any user interaction
fn enable_auto_start_silent() {
    if let Err(e) = set_autostart_registry() {
        eprintln!("❌ Failed to enable auto-start silently: {}", e);
    }
}

// === BACKGROUND INITIALIZATION ===

async fn initialize_application_async(app_handle: tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Starting background initialization...");

    // test_user_session_functions();

    // Step 1: Initialize database (SQLite + optional Postgres)
    if let Err(e) = initialize_database_async(&app_handle).await {
        eprintln!("❌ Database initialization failed: {}", e);
        let _ = app_handle.emit(
            "database-status",
            serde_json::json!({ "status": "error", "message": e.to_string() }),
        );
        return Err(e);
    }

    

    // Step 2: Start clipboard monitoring
    if let Err(e) = start_clipboard_monitoring_async(&app_handle).await {
        eprintln!("❌ Clipboard monitoring failed to start: {}", e);
        let _ = app_handle.emit(
            "clipboard-status",
            serde_json::json!({ "status": "error", "message": e.to_string() }),
        );
    }

    // Step 3: Schedule update checks (delayed)
    schedule_update_checks(app_handle).await;

    println!("✅ Background initialization completed");
    Ok(())
}

fn test_user_session_functions() {
    // Test setting the user session
    set_current_user("user123".to_string(), "org123".to_string(), "user@example.com".to_string());
    
    // Test getting the current user ID
    if let Some(user_id) = get_current_user_id() {
        println!("✅ User ID: {}", user_id);
    } else {
        println!("❌ No user ID found");
    }

    // Test getting the current organization ID
    if let Some(org_id) = get_current_organization_id() {
        println!("✅ Organization ID: {}", org_id);
    } else {
        println!("❌ No organization ID found");
    }

    // Test getting the current user email
    if let Some(email) = get_current_user_email() {
        println!("✅ User email: {}", email);
    } else {
        println!("❌ No user email found");
    }

    // Test getting the current session
    if let Some(session) = get_current_session() {
        println!("✅ Current session: {:?}", session);
    } else {
        println!("❌ No session found");
    }

    // Test clearing the current user session
    clear_current_user();

    // Test if the user is logged in after clearing the session
    if is_user_logged_in() {
        println!("✅ User is logged in");
    } else {
        println!("❌ User is not logged in");
    }
}

/// ✅ Async database initialization with offline fallback

async fn initialize_database_async(
    app_handle: &tauri::AppHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::db::sqlite_database::create_sqlite_pool;
    use crate::db::database::create_db_pool;

    println!("🔄 Initializing database connection...");

    let _ = app_handle.emit(
        "database-status",
        serde_json::json!({
            "status": "connecting",
            "message": "Connecting to database..."
        }),
    );

    // 1️⃣ Always init SQLite first (offline-first)
    let sqlite_pool = create_sqlite_pool().await?;

    let _ = app_handle.emit(
        "database-status",
        serde_json::json!({
            "status": "creating_tables",
            "message": "Setting up local database..."
        }),
    );

    let pg_pool: Option<PgPool> =
        match time::timeout(Duration::from_secs(30), create_db_pool()).await {
            Ok(Ok(pool)) => {
                println!("✅ Connected to Postgres (Neon)");
                Some(pool)
            }
            Ok(Err(e)) => {
                eprintln!("⚠️ Failed to connect to Postgres, running OFFLINE (SQLite only): {e}");
                let _ = app_handle.emit(
                    "database-status",
                    serde_json::json!({
                        "status": "offline",
                        "message": "Cloud sync unavailable, using local clipboard history only"
                    }),
                );
                None
            }
            Err(_) => {
                eprintln!("⚠️ Postgres connection TIMED OUT (3s). Running OFFLINE (SQLite only).");
                let _ = app_handle.emit(
                    "database-status",
                    serde_json::json!({
                        "status": "offline",
                        "message": "Cloud sync timed out, using local clipboard history only"
                    }),
                );
                None
            }
        };

    // 3️⃣ Store pools in state
    app_handle.manage(DbPools {
        pg: pg_pool,
        sqlite: sqlite_pool,
    });

    // 4️⃣ Mark DB ready (at least SQLite is OK)
    let state: State<'_, AppState> = app_handle.state();
    state.is_database_ready.store(true, Ordering::SeqCst);

    let _ = app_handle.emit(
        "database-status",
        serde_json::json!({
            "status": "ready",
            "message": "Local database ready"
        }),
    );

    println!("✅ Database initialized (SQLite + optional Postgres)");
    Ok(())
}


/// ✅ Async clipboard monitoring startup
async fn start_clipboard_monitoring_async(
    app_handle: &tauri::AppHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Starting clipboard monitoring...");

    let state: State<'_, AppState> = app_handle.state();

    // Wait until DB ready
    if !wait_for_database_ready(app_handle, Duration::from_secs(5)).await {
        return Err("Database not ready within timeout period".into());
    }

    // Get Postgres (Option) + SQLite
    let db_pools: State<'_, DbPools> = app_handle.state();
    let pg_pool = db_pools.pg.clone(); // Option<PgPool>
    let sqlite_pool = db_pools.sqlite.clone(); // SqlitePool

    // Mark clipboard monitoring ON
    state.is_clipboard_monitoring.store(true, Ordering::SeqCst);

    // Start actual monitoring
    let result = start_clipboard_monitoring(
        app_handle.clone(),
        pg_pool,
        sqlite_pool,
    )
    .await;

    match result {
        Ok(()) => {
            println!("✅ Clipboard monitoring started successfully");
            let _ = app_handle.emit(
                "clipboard-status",
                serde_json::json!({
                    "status": "ready",
                    "message": "Clipboard monitoring active"
                }),
            );
            Ok(())
        }
        Err(e) => {
            state.is_clipboard_monitoring.store(false, Ordering::SeqCst);
            let _ = app_handle.emit(
                "clipboard-status",
                serde_json::json!({
                    "status": "error",
                    "message": e.to_string()
                }),
            );
            Err(e)
        }
    }
}

/// ✅ Wait for database to be ready with timeout
async fn wait_for_database_ready(
    app_handle: &tauri::AppHandle,
    timeout: Duration,
) -> bool {
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        let state: State<'_, AppState> = app_handle.state();
        if state.is_database_ready.load(Ordering::SeqCst) {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    false
}

/// ✅ Schedule update checks with proper delays
async fn schedule_update_checks(app_handle: tauri::AppHandle) {
    let github_owner = get_github_owner();
    let github_repo = get_github_repo();
    let current_version = get_current_version();

    // Initial update check after app is stable
    let app_handle_1 = app_handle.clone();
    async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(15)).await;

        println!("🔍 Performing automatic update check...");
        let updater = Updater::new(github_owner.clone(), github_repo.clone(), current_version.clone());
        updater.check_and_notify(app_handle_1).await;
    });

    // Auto-update attempt after longer delay
    let app_handle_2 = app_handle.clone();
    async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(45)).await;

        println!("🔄 Attempting automatic update...");
        let mut updater = Updater::new(github_owner, github_repo, current_version);
        match updater.auto_update(app_handle_2).await {
            Ok(true) => println!("✅ Auto-update completed successfully"),
            Ok(false) => println!("✅ No updates available for auto-update"),
            Err(e) => eprintln!("❌ Auto-update failed: {}", e),
        }
    });
}

// === TRAY + UI ===

fn setup_tray_and_ui(app: &mut tauri::App) -> tauri::Result<()> {
    println!("🎨 Setting up tray icon and UI...");

    let app_handle = app.handle().clone();

    // Create tray menu
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit])?;
    let icon = app
        .default_window_icon()
        .expect("App should have a default icon")
        .clone();

    // Build tray icon
    let tray = TrayIconBuilder::new()
        .icon(icon)
        .tooltip("ClipTray — starting...")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, e| match e.id.as_ref() {
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(move |tray, ev| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = ev
            {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    if w.is_visible().unwrap_or(false) {
                        let _ = w.hide();
                        return;
                    }
                }
                if let Err(e) = position_top_right_with_padding(&app, "main", MIN_POP_H) {
                    eprintln!("Failed to position window: {}", e);
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    // Setup auto-updater
    setup_silent_auto_updater(&app.handle());

   // Update tray tooltip after a short delay (use Tauri runtime)
let app_handle_for_tooltip = app_handle.clone();
let tray_id = tray.id().clone(); // 👈 clone the ID so it is owned and 'static
async_runtime::spawn(async move {
    tokio::time::sleep(Duration::from_secs(2)).await;

    if let Some(tray) = app_handle_for_tooltip.tray_by_id(&tray_id) {
        let _ = tray.set_tooltip(Some("ClipTray — ready"));
    }
});


    println!("✅ UI setup completed");
    Ok(())
}

/// ✅ Command to check database status from frontend
#[tauri::command]
async fn check_database_status(
    app_state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    // Just read the atomic flag that initialize_database_async sets
    let ready = app_state.is_database_ready.load(Ordering::SeqCst);

    if ready {
        println!("✅ check_database_status → DB is ready");
    } else {
        println!("⏳ check_database_status → DB not ready yet");
    }

    Ok(ready)
}



/// ✅ Window positioning helper
fn position_top_right_with_padding(
    app: &tauri::AppHandle,
    win_label: &str,
    height: f64,
) -> Result<(), String> {
    ensure_main_window(app).map_err(|e| e.to_string())?;
    let window = app
        .get_webview_window(win_label)
        .ok_or("Window not found")?;
    let monitor = app
        .primary_monitor()
        .map_err(|e| e.to_string())?
        .ok_or("No monitor")?;
    let mpos = monitor.position();
    let msize = monitor.size();

    let x = mpos.x as f64 + msize.width as f64 - POP_W - 20.0;
    let y = mpos.y as f64 + 20.0;

    window
        .set_position(PhysicalPosition { x, y })
        .map_err(|e| e.to_string())?;
    window
        .set_size(PhysicalSize {
            width: POP_W,
            height,
        })
        .map_err(|e| e.to_string())?;
    window.set_always_on_top(true).map_err(|e| e.to_string())?;
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

/// ✅ Ensure main window exists
fn ensure_main_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if app.get_webview_window("main").is_some() {
        return Ok(());
    }
    tauri::WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .visible(false)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .transparent(true)
        .shadow(true)
        .inner_size(POP_W, MIN_POP_H)
        .min_inner_size(POP_W, MIN_POP_H)
        .max_inner_size(POP_W, MAX_POP_H)
        .build()?;
    Ok(())
}

fn ensure_pricing_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if app.get_webview_window("pricing").is_some() {
        return Ok(());
    }

    WebviewWindowBuilder::new(app, "pricing", WebviewUrl::App("index.html#/pricing".into()))
        .title("ClipTray — Pricing")
        .visible(false)
        .resizable(true)
        .maximizable(true)
        .minimizable(true)
        .decorations(true)
        .always_on_top(false)
        .skip_taskbar(true)
        .inner_size(500.0, 350.0)
        .min_inner_size(200.0, 50.0)
        .build()?;

    Ok(())
}


#[tauri::command]
fn open_pricing_window(app: tauri::AppHandle) -> Result<(), String> {
    ensure_pricing_window(&app).map_err(|e| e.to_string())?;

    if let Some(w) = app.get_webview_window("pricing") {
        w.show().map_err(|e| e.to_string())?;
        w.set_focus().map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// ✅ Window resize command
#[tauri::command]
fn resize_window(app: tauri::AppHandle, height: f64) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let clamped_height = height.clamp(MIN_POP_H, MAX_POP_H);
        window
            .set_size(PhysicalSize {
                width: POP_W,
                height: clamped_height,
            })
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
