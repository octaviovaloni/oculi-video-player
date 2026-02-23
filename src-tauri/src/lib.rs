use walkdir::WalkDir;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct Video {
    name: String,
    path: String,
    miniature_path: String,
    duration: f64,
}

fn get_file_name(path: &Path) -> String {
    if let Some(name) = path.file_name() {
        return name.to_string_lossy().to_string();
    } else {
        return "NAME_ERROR".to_string();
    }
}

#[tauri::command]
fn list_videos(path: String) -> Result<Vec<Video>, String> {
    println!("Processing videos in {}", path.to_string());

    let mut videos: Vec<Video> = Vec::new();

    for entry in WalkDir::new(path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() == false {continue}
        
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();

            if ext == "mp4" || ext == "mkv" {
                videos.push(Video {
                    name: get_file_name(path),
                    path: path.to_string_lossy().to_string(),
                    miniature_path: String::new(),
                    duration: 0.0
                });
            }
        }
    }

    println!("Found {} videos", videos.len());
    Ok(videos)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![list_videos])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
