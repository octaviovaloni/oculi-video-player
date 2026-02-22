use walkdir::WalkDir;

#[tauri::command]
fn list_videos(path: String) -> Result<Vec<String>, String> {
    println!("Processing videos in {}", path.to_string());

    let mut videos: Vec<String> = Vec::new();

    for entry in WalkDir::new(path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() == false {continue}
        
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();

            if ext == "mp4" || ext == "mkv" {
                println!("Video: {}", path.display());
            }
        }
    }

    Ok(videos)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![list_videos])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
