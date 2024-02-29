use std::thread;
use tauri::Manager;

mod api;

#[tauri::command]
fn add_file(path: &str) {
	api::add_file(path)
}

#[tauri::command]
fn clear_files() {
	api::clear_files()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![add_file, clear_files])
        .setup(|app| {
        	// TODO: _up_ issue
      		let resource_path = app.path().resolve("_up_/dist/index.html", tauri::path::BaseDirectory::Resource).unwrap().display().to_string();
        	tauri::async_runtime::spawn(async move {
				let server_handle = thread::spawn(move || {
					api::start(&resource_path).unwrap();
				});

				// Wait for the server thread to finish
    			server_handle.join().unwrap();
         	});
            Ok(())
          })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
