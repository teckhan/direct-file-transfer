use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::mpsc;
use tokio::signal::ctrl_c;
use actix_web::{get, web, App, HttpServer, HttpRequest, HttpResponse, Responder, main};
use actix_files as fs;
use uuid::Uuid;
use serde_json::json;
use lazy_static::lazy_static;
use regex::Regex;
use zip::write::FileOptions;
use zip::ZipWriter;
use local_ip_address::local_ip;

// #region file list states
lazy_static! {
    static ref FILE_LIST: Mutex<HashMap<String, Uuid>> = Mutex::new(HashMap::new());
}
pub fn add_file(path: &str) {
    let mut file_list = FILE_LIST.lock().unwrap();
    let id = Uuid::new_v4();
    file_list.entry(String::from(path)).or_insert(id);
}
pub fn clear_files() {
	FILE_LIST.lock().unwrap().clear();
}
// #endregion

fn extract_filename(path: &str) -> String {
    let re = Regex::new(r"/([^/]+)$").unwrap();
    if let Some(captures) = re.captures(path) {
        if let Some(filename) = captures.get(1) {
            return filename.as_str().to_string();
        }
    }
    path.to_string()
}

// #region endpoints
#[get("/ip")]
async fn get_ip() -> impl Responder {
   	let my_local_ip = local_ip().unwrap();
    format!("{}", my_local_ip)
}

#[get("/client-ip")] // TODO: remove
async fn get_client_ip(req: HttpRequest) -> impl Responder {
    let remote_addr = req.connection_info().realip_remote_addr().map(|addr| addr.to_string());

    match remote_addr {
        Some(addr) => HttpResponse::Ok().body(addr),
        None => HttpResponse::Ok().body("Unavailable"), // Informative message when IP cannot be retrieved
    }
}

#[get("/list")]
async fn list() -> impl Responder {
  let file_list = FILE_LIST.lock().unwrap(); // Acquire lock for safety
  let json_data = json!(file_list.iter()
    .map(|(path, id)| (extract_filename(path), id.to_string()))
    .collect::<HashMap<String, String>>());

    web::Json(json_data) // Return the data as JSON
}

#[get("/dl/{id}")]
async fn download(id: web::Path<String>) -> impl Responder {
	let file_list = FILE_LIST.lock().unwrap();

    // Attempt to parse the UUID from the path
    let uuid = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => return HttpResponse::BadRequest().body("Invalid UUID format"),
    };

    // Find the corresponding file path for the UUID
    let path = match file_list.iter().find_map(|(path, u)| if *u == uuid { Some(path) } else { None }) {
        Some(p) => p,
        None => return HttpResponse::NotFound().body("File not found"),
    };

    // Attempt to open and read the file
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return HttpResponse::InternalServerError().body("Error opening file"),
    };

    let mut contents = Vec::new();
    if let Err(_) = file.read_to_end(&mut contents) {
        return HttpResponse::InternalServerError().body("Error reading file");
    }

    // Get the original filename from the path
    let filename = match std::path::Path::new(&path).file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => "file".to_string(),
    };

    // Create a response with the file contents
    let response = HttpResponse::Ok()
        .append_header(("Content-Type", "application/octet-stream"))
        .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
        .body(web::Bytes::from(contents));

    response
}

#[get("/dl")]
async fn download_all() -> HttpResponse {
	let file_list = FILE_LIST.lock().unwrap();
    let mut zip_buffer = Vec::new();  // Separate buffer for writing zip data
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut zip_buffer));

        let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

        for (path, _) in file_list.iter() {
            let filename = std::path::Path::new(path).file_name().unwrap().to_string_lossy();

            if let Ok(mut file) = File::open(path) {
                let mut contents = Vec::new();
                if let Err(_) = file.read_to_end(&mut contents) {
                    continue;
                }
                if let Err(_) = zip.start_file(filename.clone(), options) {
                    continue;
                }
                if let Err(_) = zip.write_all(&contents) {
                    continue;
                }
            }
        }
    } // zip_writer goes out of scope here, allowing zip_buffer to be borrowed again

    // Reset the cursor to the beginning
    let mut cursor = Cursor::new(zip_buffer);
    cursor.seek(SeekFrom::Start(0)).unwrap();

    HttpResponse::Ok()
        .append_header(("Content-Type", "application/zip"))
        .append_header(("Content-Disposition", "attachment; filename=\"files.zip\""))
        .body(web::Bytes::copy_from_slice(&cursor.into_inner()))
}
// #endregion

#[main]
pub async fn start(resource_path: &str) -> std::io::Result<()> {
    let (tx, mut rx) = mpsc::channel(1); // Create a channel for shutdown signal

    let resource_path = std::sync::Arc::new(resource_path.to_owned());

    tokio::spawn(async move {
        if let Err(err) = ctrl_c().await {
            eprintln!("Error receiving Ctrl-C: {}", err);
        }
        tx.send(()).await.unwrap(); // Send shutdown signal
    });

    HttpServer::new(move || {
    	let resource_path = resource_path.clone();
	    App::new()
        	.service(fs::Files::new("/", resource_path.clone().as_ref()).show_files_listing().index_file("index.html").use_last_modified(true))
	    	.service(get_ip)
	    	.service(get_client_ip)
	    	.service(list)
	     	.service(download)
	     	.service(download_all)
    })
    .bind(("0.0.0.0", 8080))? // TODO: tweak to 80
    .bind(("::1", 8080))?
    .run()
    .await?;

    rx.recv().await.unwrap(); // Wait for shutdown signal
    Ok(())
}
