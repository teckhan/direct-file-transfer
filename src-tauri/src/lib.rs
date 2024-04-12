use tauri::{AppHandle, Manager};
use local_ip_address::local_ip;
use public_ip;

mod api;

// the payload type must implement `Serialize` and `Clone`.
#[derive(Clone, serde::Serialize)]
struct FileAddedPayload {
	id: String,
	path: String,
}

#[tauri::command]
fn add_file(app: AppHandle, path: &str) {
	let file_id = api::add_file(path);
	app.emit("file-added", FileAddedPayload { id: file_id.to_string(), path: path.to_string() }).unwrap();
}

#[tauri::command]
fn clear_files(app: AppHandle) {
	app.emit("cleared-all", ()).unwrap();
	api::clear_files()
}

#[tauri::command]
fn get_local_ip() -> std::net::IpAddr {
	local_ip().unwrap()
}

#[tauri::command]
async fn get_host_public_up() -> std::net::IpAddr {
	public_ip::addr().await.unwrap()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![add_file, clear_files, get_local_ip, get_host_public_up])
        .setup(|app| {
        	// TODO: _up_ issue
      		let resource_path = app.path().resolve("_up_/dist", tauri::path::BaseDirectory::Resource).unwrap().display().to_string();
      		let desktop_path = app.path().resolve("", tauri::path::BaseDirectory::Desktop).unwrap().display().to_string();
        	tauri::async_runtime::spawn(async move {
				let server_handle = std::thread::spawn(move || {
					api::start(&resource_path, &desktop_path).unwrap();
				});

				// Wait for the server thread to finish
    			server_handle.join().unwrap();
         	});
            Ok(())
          })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
