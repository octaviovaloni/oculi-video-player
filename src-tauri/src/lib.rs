use tauri::{AppHandle, Manager};
use tauri::path::BaseDirectory;
use walkdir::WalkDir;
use serde::Serialize;
use std::path::Path;
use std::process::Command;

#[derive(Serialize)]
struct Video {
    name: String,
    path: String,
    miniature_path: String,
    duration: f64,
}

fn get_ffprobe(app: &AppHandle) -> Result<std::path::PathBuf, String> {
  app.path()
    .resolve("bin/ffprobe.exe", BaseDirectory::Resource)
    .map_err(|e| e.to_string())
}

fn get_ffmpeg(app: &AppHandle) -> Result<std::path::PathBuf, String> {
  app.path()
    .resolve("bin/ffmpeg.exe", BaseDirectory::Resource)
    .map_err(|e| e.to_string())
}

fn get_file_name(path: &Path) -> String {
    if let Some(name) = path.file_name() {
        return name.to_string_lossy().to_string();
    } else {
        return "NAME_ERROR".to_string();
    }
}

fn get_video_duration(ffprobe: &std::path::PathBuf, video_path: &str) -> Result<f64, String> {
    let output = Command::new(ffprobe)
        .args([
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            video_path,
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(format!("Error ocurred while retrieving duration of '{}'", video_path));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| e.to_string())?;

    let duration = stdout
        .trim()
        .parse::<f64>()
        .map_err(|e| e.to_string())?;

    Ok(duration)
}

#[tauri::command]
fn list_videos(app: AppHandle, path: String) -> Result<Vec<Video>, String> {
    println!("Processing videos in {}", path.to_string());

    let mut videos: Vec<Video> = Vec::new();
    let ffprobe: std::path::PathBuf = get_ffprobe(&app)?;
    let mut actual_video_duration: f64;
    let mut actual_video_path: String;
    let mut actual_video_name: String;

    for entry in WalkDir::new(path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() == false {continue}
        
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();

            if ext == "mp4" || ext == "mkv" {
                actual_video_path = path.to_string_lossy().to_string();
                println!("Processing {}...", actual_video_path);
                actual_video_duration = get_video_duration(
                    &ffprobe,
                    path.to_str().expect(&format!("Error ocurred while converting Path -> str in '{}' Path object...", actual_video_path))
                )?;
                actual_video_name = get_file_name(path);

                videos.push(Video {
                    name: actual_video_name,
                    path: actual_video_path,
                    miniature_path: String::new(),
                    duration: actual_video_duration
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
