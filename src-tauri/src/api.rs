use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use tokio::sync::mpsc;
use tokio::signal::ctrl_c;
use actix_web::{get, post, web, App, HttpServer, HttpResponse, Responder, main};
use actix_files as fs;
use actix_cors::Cors;
use actix_multipart::form::{tempfile::TempFile,MultipartForm, MultipartFormConfig};
use uuid::Uuid;
use serde_json::json;
use lazy_static::lazy_static;
use regex::Regex;
use zip::write::FileOptions;
use zip::ZipWriter;

mod broadcast;
use self::broadcast::{Broadcaster, Message};

struct AppData {
	desktop_path: String
}

// #region file list states
lazy_static! {
    static ref FILE_LIST: Mutex<HashMap<String, Uuid>> = Mutex::new(HashMap::new());
    static ref BROADCASTER: Mutex<Arc<Broadcaster>> = Mutex::new(Broadcaster::create());
}
fn extract_filename(path: &str) -> String {
    let re = Regex::new(r"/([^/]+)$").unwrap();
    if let Some(captures) = re.captures(path) {
        if let Some(filename) = captures.get(1) {
            return filename.as_str().to_string();
        }
    }
    path.to_string()
}

pub fn add_file(path: &str) -> Uuid {
    let mut file_list = FILE_LIST.lock().unwrap();
    let mut id = Uuid::new_v4();
    file_list.entry(String::from(path)).or_insert(id);

    id = match file_list.iter().find_map(|(p, u)| if *p == path { Some(u) } else { None }) {
        Some(u) => *u,
        None => id
    };

    BROADCASTER.lock().unwrap().broadcast_sync(Message {
	    action: "file-added".to_string(),
		payload: json!({
	        "id": id.to_string(),
	        "file_name": extract_filename(path)
	    }).to_string()
    });

    id
}
pub fn clear_files() {
	FILE_LIST.lock().unwrap().clear();

	BROADCASTER.lock().unwrap().broadcast_sync(Message {
		action: "all-files-cleared".to_string(),
		payload: "".to_string()
	});
}
// #endregion

// #region endpoints
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
async fn download_all() -> impl Responder {
	let file_list = FILE_LIST.lock().unwrap();

	if file_list.len() == 1 {
        let (path, _) = file_list.iter().next().unwrap();
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
        return HttpResponse::Ok()
            .append_header(("Content-Type", "application/octet-stream"))
            .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
            .body(web::Bytes::from(contents));
    } else {
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

	    return HttpResponse::Ok()
	        .append_header(("Content-Type", "application/zip"))
	        .append_header(("Content-Disposition", "attachment; filename=\"files.zip\""))
	        .body(web::Bytes::copy_from_slice(&cursor.into_inner()))
    }
}

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "file")]
    files: Vec<TempFile>,
}

#[post("/upload")]
async fn upload(data: web::Data<AppData>,  MultipartForm(form): MultipartForm<UploadForm>) -> impl Responder {
    for f in form.files {
        let path = format!("{}{}",data.desktop_path.to_string(), f.file_name.unwrap());
        f.file.persist(path).unwrap();
    }

   HttpResponse::Ok()
}

#[get("/events")]
async fn event_stream() -> impl Responder {
	BROADCASTER.lock().unwrap().new_client().await
}

#[main]
pub async fn start(resource_path: &str, desktop_path: &str) -> std::io::Result<()> {
    let (tx, mut rx) = mpsc::channel(1); // Create a channel for shutdown signal

    let resource_path = Arc::new(resource_path.to_owned());
    let desktop_path = Arc::new(desktop_path.to_owned());

    tokio::spawn(async move {
        if let Err(err) = ctrl_c().await {
            eprintln!("Error receiving Ctrl-C: {}", err);
        }
        tx.send(()).await.unwrap(); // Send shutdown signal
    });

    HttpServer::new(move || {
    	let resource_path = resource_path.clone();
    	let desktop_path = desktop_path.clone();
    	let cors = Cors::default().allow_any_method().allow_any_header().allow_any_origin().send_wildcard();

	    App::new()
			.app_data(MultipartFormConfig::default()
            	.total_limit(100 * 1024 * 1024 * 1024) // 100GB: https://docs.rs/actix-multipart/latest/actix_multipart/form/struct.MultipartFormConfig.html
         	)
	        .app_data(web::Data::new(AppData { desktop_path: desktop_path.to_string() }))
			.wrap(cors)
	    	.service(list)
	     	.service(download)
	     	.service(download_all)
	     	.service(upload)
			.service(event_stream)
        	.service(fs::Files::new("/", resource_path.clone().as_ref()).show_files_listing().index_file("index.html").use_last_modified(true))
    })
    .bind(("0.0.0.0", 4321))?
    .bind(("::1", 4321))?
    .run()
    .await?;

    rx.recv().await.unwrap(); // Wait for shutdown signal
    Ok(())
}
