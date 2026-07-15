use std::collections::HashMap;
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

use actix_cors::Cors;
use actix_files::NamedFile;
use actix_multipart::form::{tempfile::TempFile, MultipartForm, MultipartFormConfig};
use actix_web::error::{ErrorInternalServerError, ErrorNotFound};
use actix_web::http::header::{ContentDisposition, DispositionParam, DispositionType};
use actix_web::{get, main, post, web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use serde_json::json;
use tauri::Emitter;
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

mod broadcast;
use self::broadcast::{Broadcaster, Message};

#[derive(Clone, Serialize)]
pub struct SharedFile {
    pub id: String,
    pub file_name: String,
    pub size: u64,
    #[serde(skip)]
    pub path: PathBuf,
}

static FILE_LIST: LazyLock<Mutex<Vec<SharedFile>>> = LazyLock::new(|| Mutex::new(Vec::new()));
static SAVE_DIR: LazyLock<Mutex<PathBuf>> = LazyLock::new(|| Mutex::new(PathBuf::new()));
static BROADCASTER: LazyLock<std::sync::Arc<Broadcaster>> = LazyLock::new(Broadcaster::create);

fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

fn file_size(path: &Path) -> u64 {
    std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0)
}

pub fn set_save_dir(dir: PathBuf) {
    *SAVE_DIR.lock().unwrap() = dir;
}

pub fn save_dir() -> PathBuf {
    SAVE_DIR.lock().unwrap().clone()
}

// #region file list states
pub fn add_file(path: &str) -> SharedFile {
    let path = PathBuf::from(path);
    let mut file_list = FILE_LIST.lock().unwrap();

    if let Some(existing) = file_list.iter().find(|file| file.path == path) {
        return existing.clone();
    }

    let file = SharedFile {
        id: Uuid::new_v4().to_string(),
        file_name: display_name(&path),
        size: file_size(&path),
        path,
    };
    file_list.push(file.clone());
    drop(file_list);

    BROADCASTER.broadcast_sync(Message {
        action: "file-added".to_string(),
        payload: json!({
            "id": file.id,
            "file_name": file.file_name,
            "size": file.size,
        })
        .to_string(),
    });

    file
}

pub fn remove_file(id: &str) -> bool {
    let mut file_list = FILE_LIST.lock().unwrap();
    let count_before = file_list.len();
    file_list.retain(|file| file.id != id);
    let removed = file_list.len() != count_before;
    drop(file_list);

    if removed {
        BROADCASTER.broadcast_sync(Message {
            action: "file-removed".to_string(),
            payload: json!({ "id": id }).to_string(),
        });
    }

    removed
}

pub fn clear_files() {
    FILE_LIST.lock().unwrap().clear();

    BROADCASTER.broadcast_sync(Message {
        action: "all-files-cleared".to_string(),
        payload: "".to_string(),
    });
}
// #endregion

/// If `file_name` is already taken in `dir`, append " (n)" before the
/// extension until the name is free, so uploads never overwrite silently.
fn unique_save_path(dir: &Path, file_name: &str) -> PathBuf {
    let candidate = dir.join(file_name);
    if !candidate.exists() {
        return candidate;
    }

    let name = Path::new(file_name);
    let stem = name
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| file_name.to_string());
    let extension = name.extension().map(|e| e.to_string_lossy().into_owned());

    for n in 1u32.. {
        let numbered = match &extension {
            Some(ext) => format!("{stem} ({n}).{ext}"),
            None => format!("{stem} ({n})"),
        };
        let candidate = dir.join(numbered);
        if !candidate.exists() {
            return candidate;
        }
    }

    unreachable!("ran out of candidate file names")
}

/// Same idea as `unique_save_path`, but for entry names inside the zip
/// archive, where collisions come from shared files with equal basenames.
fn unique_zip_name(used: &mut HashMap<String, u32>, file_name: &str) -> String {
    let n = used.entry(file_name.to_string()).or_insert(0);
    *n += 1;
    if *n == 1 {
        return file_name.to_string();
    }

    let name = Path::new(file_name);
    let stem = name
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| file_name.to_string());
    match name.extension() {
        Some(ext) => format!("{stem} ({}).{}", *n - 1, ext.to_string_lossy()),
        None => format!("{stem} ({})", *n - 1),
    }
}

fn attachment(file_name: &str) -> ContentDisposition {
    ContentDisposition {
        disposition: DispositionType::Attachment,
        parameters: vec![DispositionParam::Filename(file_name.to_string())],
    }
}

/// Open a shared file as a streaming response. `NamedFile` streams from disk
/// in chunks and supports HTTP Range requests, so large files never have to
/// fit in memory.
async fn named_file_response(entry: &SharedFile) -> actix_web::Result<NamedFile> {
    let file = NamedFile::open_async(&entry.path)
        .await
        .map_err(|_| ErrorInternalServerError("Error opening file"))?;
    Ok(file.set_content_disposition(attachment(&entry.file_name)))
}

// #region endpoints
#[get("/list")]
async fn list() -> impl Responder {
    let file_list = FILE_LIST.lock().unwrap();
    web::Json(file_list.clone())
}

#[get("/dl/{id}")]
async fn download(id: web::Path<String>) -> actix_web::Result<NamedFile> {
    let entry = FILE_LIST
        .lock()
        .unwrap()
        .iter()
        .find(|file| file.id == *id)
        .cloned();

    match entry {
        Some(entry) => named_file_response(&entry).await,
        None => Err(ErrorNotFound("File not found")),
    }
}

#[get("/dl")]
async fn download_all() -> actix_web::Result<NamedFile> {
    let entries: Vec<SharedFile> = FILE_LIST.lock().unwrap().clone();

    match entries.len() {
        0 => Err(ErrorNotFound("No files are being shared")),
        1 => named_file_response(&entries[0]).await,
        _ => {
            // Build the archive in an unnamed temp file (spooled to disk, not
            // RAM) on a blocking thread, then stream it back with NamedFile.
            let file = web::block(move || -> std::io::Result<std::fs::File> {
                let mut tmp = tempfile::tempfile()?;
                {
                    let mut zip = ZipWriter::new(&mut tmp);
                    let options = SimpleFileOptions::default()
                        .compression_method(zip::CompressionMethod::Stored);
                    let mut used_names = HashMap::new();

                    for entry in &entries {
                        let name = unique_zip_name(&mut used_names, &entry.file_name);
                        let Ok(mut src) = std::fs::File::open(&entry.path) else {
                            continue; // file was moved/deleted since being shared
                        };
                        zip.start_file(name, options)?;
                        std::io::copy(&mut src, &mut zip)?;
                    }
                    zip.finish()?;
                }
                tmp.seek(SeekFrom::Start(0))?;
                Ok(tmp)
            })
            .await
            .map_err(|_| ErrorInternalServerError("Error building archive"))?
            .map_err(|_| ErrorInternalServerError("Error building archive"))?;

            let named = NamedFile::from_file(file, "files.zip")
                .map_err(|_| ErrorInternalServerError("Error building archive"))?;
            Ok(named.set_content_disposition(attachment("files.zip")))
        }
    }
}

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "file")]
    files: Vec<TempFile>,
}

/// `NamedTempFile::persist` is a rename under the hood, which fails when the
/// OS temp dir sits on a different filesystem than the destination (common
/// with tmpfs on Linux) — fall back to a copy in that case.
fn persist_temp_file(file: tempfile::NamedTempFile, dest: &Path) -> std::io::Result<()> {
    match file.persist(dest) {
        Ok(_) => Ok(()),
        Err(err) => {
            std::fs::copy(err.file.path(), dest)?;
            Ok(())
        }
    }
}

#[post("/upload")]
async fn upload(
    app: web::Data<tauri::AppHandle>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> actix_web::Result<impl Responder> {
    let save_dir = save_dir();
    let mut saved = Vec::new();

    for f in form.files {
        // keep only the final path component so a crafted name can't escape the save dir
        let file_name = f
            .file_name
            .as_deref()
            .and_then(|name| Path::new(name).file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let path = unique_save_path(&save_dir, &file_name);

        persist_temp_file(f.file, &path).map_err(|err| {
            ErrorInternalServerError(format!("Failed to save \"{file_name}\": {err}"))
        })?;

        saved.push(json!({
            "name": display_name(&path),
            "path": path.display().to_string(),
            "size": file_size(&path),
        }));
    }

    for info in &saved {
        let _ = app.emit("file-received", info);
    }

    Ok(HttpResponse::Ok().json(json!({ "saved": saved })))
}

#[get("/events")]
async fn event_stream() -> impl Responder {
    BROADCASTER.new_client().await
}
// #endregion

#[main]
pub async fn start(app: tauri::AppHandle, resource_path: PathBuf) -> std::io::Result<()> {
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_method()
            .allow_any_header()
            .allow_any_origin()
            .send_wildcard();

        App::new()
            .app_data(
                MultipartFormConfig::default().total_limit(100 * 1024 * 1024 * 1024), // 100GB: https://docs.rs/actix-multipart/latest/actix_multipart/form/struct.MultipartFormConfig.html
            )
            .app_data(web::Data::new(app.clone()))
            .wrap(cors)
            .service(list)
            .service(download)
            .service(download_all)
            .service(upload)
            .service(event_stream)
            .service(
                actix_files::Files::new("/", &resource_path)
                    .index_file("index.html")
                    .use_last_modified(true),
            )
    })
    .bind(("0.0.0.0", 8080))?
    .bind(("::1", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_name_returns_basename() {
        assert_eq!(display_name(Path::new("/home/user/notes.txt")), "notes.txt");
    }

    #[test]
    fn unique_save_path_numbers_collisions() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), b"x").unwrap();
        std::fs::write(dir.path().join("a (1).txt"), b"x").unwrap();

        assert_eq!(
            unique_save_path(dir.path(), "a.txt"),
            dir.path().join("a (2).txt")
        );
        assert_eq!(
            unique_save_path(dir.path(), "b.txt"),
            dir.path().join("b.txt")
        );
    }

    #[test]
    fn unique_zip_name_numbers_duplicates() {
        let mut used = HashMap::new();
        assert_eq!(unique_zip_name(&mut used, "a.txt"), "a.txt");
        assert_eq!(unique_zip_name(&mut used, "a.txt"), "a (1).txt");
        assert_eq!(unique_zip_name(&mut used, "a.txt"), "a (2).txt");
        assert_eq!(unique_zip_name(&mut used, "b"), "b");
        assert_eq!(unique_zip_name(&mut used, "b"), "b (1)");
    }
}
