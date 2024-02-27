use std::thread;
mod api;

#[tauri::command]
fn add_file(path: &str) {
	api::add_file(path)
}

#[tauri::command]
fn clear_files() {
	api::clear_files()
}

fn run_server() {
   if let Err(e) = api::start() {
       eprintln!("Error Running Api Server: {}", e);
   }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	let server_handle = thread::spawn(run_server);

	tauri::Builder::default()
    	// .plugin(tauri_plugin_localhost::Builder::new("7070").build()) // TODO: cannot external ip
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![add_file, clear_files])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    // Wait for the server thread to finish
    server_handle.join().unwrap();
}
