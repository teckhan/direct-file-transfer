use std::path::PathBuf;
use std::sync::Mutex;

use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

mod api;

/// Set when the embedded HTTP server fails to start (e.g. port 8080 already
/// taken), so the frontend can surface it even if it missed the event.
static SERVER_ERROR: Mutex<Option<String>> = Mutex::new(None);

// the payload type must implement `Serialize` and `Clone`.
#[derive(Clone, Serialize)]
struct FileAddedPayload {
    id: String,
    path: String,
    file_name: String,
    size: u64,
}

#[tauri::command]
fn add_file(app: AppHandle, path: &str) {
    let file = api::add_file(path);
    let _ = app.emit(
        "file-added",
        FileAddedPayload {
            id: file.id,
            path: path.to_string(),
            file_name: file.file_name,
            size: file.size,
        },
    );
}

#[tauri::command]
fn remove_file(app: AppHandle, id: String) {
    if api::remove_file(&id) {
        let _ = app.emit("file-removed", id);
    }
}

#[tauri::command]
fn clear_files(app: AppHandle) {
    let _ = app.emit("cleared-all", ());
    api::clear_files()
}

#[tauri::command]
fn get_local_ip() -> Result<String, String> {
    local_ip()
        .map(|ip| ip.to_string())
        .map_err(|err| err.to_string())
}

#[tauri::command]
async fn get_public_ip() -> Result<String, String> {
    public_ip::addr()
        .await
        .map(|ip| ip.to_string())
        .ok_or_else(|| "Could not determine public IP".to_string())
}

#[tauri::command]
fn get_server_error() -> Option<String> {
    SERVER_ERROR.lock().unwrap().clone()
}

#[tauri::command]
fn get_save_dir() -> String {
    api::save_dir().display().to_string()
}

#[tauri::command]
fn set_save_dir(app: AppHandle, path: String) -> Result<String, String> {
    let path = PathBuf::from(path);
    if !path.is_dir() {
        return Err(format!("\"{}\" is not a directory", path.display()));
    }

    api::set_save_dir(path.clone());
    save_config(
        &app,
        &Config {
            save_dir: Some(path.clone()),
        },
    );

    Ok(path.display().to_string())
}

#[tauri::command]
fn reveal_in_folder(path: String) -> Result<(), String> {
    tauri_plugin_opener::reveal_item_in_dir(path).map_err(|err| err.to_string())
}

// #region config
#[derive(Default, Serialize, Deserialize)]
struct Config {
    save_dir: Option<PathBuf>,
}

fn config_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|dir| dir.join("config.json"))
}

fn load_config(app: &AppHandle) -> Config {
    config_path(app)
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn save_config(app: &AppHandle, config: &Config) {
    let Some(path) = config_path(app) else { return };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(raw) = serde_json::to_string_pretty(config) {
        let _ = std::fs::write(path, raw);
    }
}
// #endregion

// Where received files are saved. Desktop is preferred, but it doesn't exist on
// every platform (mobile, headless Linux), so fall back to the first directory
// that actually exists: Desktop -> Downloads -> Documents -> app data.
fn resolve_save_dir(app: &tauri::App) -> PathBuf {
    let path = app.path();
    let candidates = [
        path.desktop_dir(),
        path.download_dir(),
        path.document_dir(),
        path.app_data_dir(),
    ];
    for dir in candidates.into_iter().flatten() {
        if dir.is_dir() {
            return dir;
        }
    }

    let fallback = app
        .path()
        .app_data_dir()
        .expect("no writable save directory available");
    std::fs::create_dir_all(&fallback).expect("failed to create save directory");
    fallback
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    // Focus the existing window instead of starting a second instance (which
    // would also lose the fight over port 8080). Must be the first plugin.
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }));
    }

    builder
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            add_file,
            remove_file,
            clear_files,
            get_local_ip,
            get_public_ip,
            get_server_error,
            get_save_dir,
            set_save_dir,
            reveal_in_folder
        ])
        .setup(|app| {
            let resource_path = app
                .path()
                .resolve("dist", tauri::path::BaseDirectory::Resource)?;

            let configured_save_dir = load_config(app.handle())
                .save_dir
                .filter(|dir| dir.is_dir());
            api::set_save_dir(configured_save_dir.unwrap_or_else(|| resolve_save_dir(app)));

            let handle = app.handle().clone();
            std::thread::spawn(move || {
                if let Err(err) = api::start(handle.clone(), resource_path) {
                    let message = format!("Failed to start the file server on port 8080: {err}");
                    *SERVER_ERROR.lock().unwrap() = Some(message.clone());
                    let _ = handle.emit("server-error", message);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
