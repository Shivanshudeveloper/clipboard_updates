#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod commands;
mod command;
mod auth; // ✅ added missing semicolon here!
mod session;
mod updater;

use tauri::{
    Manager, Emitter,
    menu::{Menu, MenuItem}, 
    tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent},
    PhysicalPosition, PhysicalSize,
};
use tauri_utils::config::WebviewUrl;
use std::time::Duration;
use command::setup_silent_auto_updater;
use command::{
    get_all_entries, get_recent_entries, get_entry_by_id, delete_entry, 
    update_entry_content, search_entries, update_entry, login_user, signup_user,debug_get_specific_fields,get_my_entries
    ,get_organization_tags,
            create_tag,
            update_tag,
            delete_tag,
            get_tag_stats,
        assign_tag_to_entry,
    remove_tag_from_entry,
    purge_unpinned_entries,
purge_entries_older_than,
purge_unpinned_older_than,
get_purge_cadence_options,
get_current_purge_settings,
update_purge_cadence,
logout_user,
update_auto_purge_settings,
check_for_updates,
install_update,
download_update,
install_downloaded_update,
cancel_update,
};
use tauri::async_runtime::Mutex;
use crate::updater::Updater; 
use crate::commands::clipboard::start_clipboard_monitoring;

use crate::db::database::{create_db_pool, create_tables}; // ✅ use explicit path

const POP_W: f64 = 460.0;
const MIN_POP_H: f64 = 850.0;
const MAX_POP_H: f64 = 900.0;

#[tokio::main]
async fn main() {
    println!("🚀 Initializing ClipTray...");

    // ✅ Create and initialize database pool
    let db_pool = match create_db_pool().await {
        Ok(pool) => {
            println!("✅ Database connected & tables created");
            if let Err(e) = create_tables(&pool).await {
                eprintln!("❌ Failed to create tables: {}", e);
            } else {
                println!("✅ Tables are created or already exist.");
            }

            pool
        }
        Err(e) => {
            eprintln!("❌ Database setup failed: {}", e);
            std::process::exit(1);
        }
    };

    tauri::Builder::default()
        .manage(db_pool.clone())
        .manage(Mutex::new(Option::<Updater>::None))
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let pool_for_test = db_pool.clone();
             let app_handle_for_clipboard = app.handle().clone();
            let db_pool_for_clipboard = db_pool.clone();



            // ✅ Re-test DB connection after setup
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(2)).await;
                if let Err(e) = sqlx::query("SELECT 1")
                    .execute(&pool_for_test)
                    .await
                {
                    eprintln!("❌ DB connection test failed: {}", e);
                } else {
                    println!("🟢 DB connection test passed");
                }
            });

            tokio::spawn(async move {
                println!("📋 Starting clipboard monitoring automatically...");
                match start_clipboard_monitoring(app_handle_for_clipboard, db_pool_for_clipboard).await {
                    Ok(()) => println!("✅ Clipboard monitoring started successfully"),
                    Err(e) => eprintln!("❌ Clipboard monitoring failed to start: {}", e),
                }
            });

            // Example async Firebase or background tasks can go here

            // Create tray + menu
            let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings, &quit])?;
            let icon = app.default_window_icon().unwrap().clone();

            TrayIconBuilder::new()
                .icon(icon)
                .tooltip("ClipTray — running")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, e| {
                    match e.id.as_ref() {
                        "quit" => app.exit(0),
                        "settings" => {
                            if let Err(err) = open_tags_window(app.clone()) {
                                eprintln!("Failed to open settings: {}", err);
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(move |tray, ev| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = ev {
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

            setup_silent_auto_updater(&app.handle());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_my_entries,
            debug_get_specific_fields,
            get_all_entries,
            get_recent_entries,
            get_entry_by_id,
            delete_entry,
            update_entry_content,
            update_entry,
            search_entries,
            login_user,
            logout_user,
            signup_user,
            get_organization_tags,//Tags
            create_tag,
            update_tag,
            delete_tag,
            get_tag_stats,
            assign_tag_to_entry,
            remove_tag_from_entry,
            purge_entries_older_than,
            purge_unpinned_entries,
            purge_unpinned_older_than,
            get_purge_cadence_options,
            get_current_purge_settings,
            update_purge_cadence,
            update_auto_purge_settings,
            check_for_updates,
            install_update,
            download_update,
install_downloaded_update,
cancel_update,
            open_tags_window,
            resize_window,
            send_test_event,
            commands::editor::open_in_notepad_and_wait,
            test_db_connection,
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
        .expect("❌ Error while running Tauri app");
}

// ✅ verify DB from frontend (or test)
#[tauri::command]
async fn test_db_connection(pool: tauri::State<'_, sqlx::PgPool>) -> Result<String, String> {
    sqlx::query("SELECT 1")
        .fetch_one(&*pool)
        .await
        .map(|_| "✅ Database connection working!".to_string())
        .map_err(|e| format!("Database connection failed: {}", e))
}

// === Utility window + positioning functions remain unchanged ===
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

    window.set_position(PhysicalPosition { x, y }).map_err(|e| e.to_string())?;
    window
        .set_size(PhysicalSize { width: POP_W, height })
        .map_err(|e| e.to_string())?;
    window.set_always_on_top(true).map_err(|e| e.to_string())?;
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

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

#[tauri::command]
fn resize_window(app: tauri::AppHandle, height: f64) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let clamped_height = height.clamp(MIN_POP_H, MAX_POP_H);
        window
            .set_size(PhysicalSize { width: POP_W, height: clamped_height })
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn send_test_event(app: tauri::AppHandle, message: String) -> Result<(), String> {
    println!("Manual test event triggered: {}", message);
    app.emit("clipboard-update", format!("MANUAL_TEST: {}", message))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn open_tags_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window("tags_window") {
        if existing.is_visible().unwrap_or(false) {
            let _ = existing.set_focus();
            return Ok(());
        } else {
            let _ = existing.close();
        }
    }
    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "tags_window",
        WebviewUrl::App("index.html#/settings".into()),
    )
    .inner_size(350.0, 300.0)
    .resizable(false)
    .maximizable(false)
    .minimizable(false)
    .skip_taskbar(true)
    .decorations(false)
    .always_on_top(true)
    .transparent(false)
    .build()
    .map_err(|e| e.to_string())?;
    let app_handle = app.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            let _ = app_handle
                .get_webview_window("tags_window")
                .and_then(|w| w.hide().ok());
        }
    });
    Ok(())
}
