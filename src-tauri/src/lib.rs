use tauri::{AppHandle, Manager};
use tauri::path::BaseDirectory;
use walkdir::WalkDir;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;
use std::env;
use std::fs;
use std::error::Error;

#[derive(Serialize)]
struct Video {
    name: String,
    path: String,
    thumbnail_path: String,
    duration: f64,
}

fn get_ffprobe(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path().resolve("bin/ffprobe.exe", BaseDirectory::Resource)
    .map_err(|e| {
        format!("Error ocurred while accessing ffprobe.exe... ({})", e)
    })
}

fn get_ffmpeg(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path().resolve("bin/ffmpeg.exe", BaseDirectory::Resource)
    .map_err(|e| e.to_string())
}

fn is_a_video(path: &Path) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => {
            matches!(ext.to_lowercase().as_str(),
                "mp4" | "mkv" | "avi" | "mov" | "webm" | "flv" | "wmv" | "mpeg"
            )
        }
        None => false
    }
}

fn get_file_name(path: &Path) -> String {
    if let Some(name) = path.file_name() { return name.to_string_lossy().to_string() } else { return "NAME_ERROR".to_string() }
}

fn get_video_duration(ffprobe: &PathBuf, video_path: &Path) -> Result<f64, String> {
    let video_path_str: &str;

    match video_path.to_str() {
        Some(result) => video_path_str = result,
        None => return Err(format!("Error ocurred while retrieving duration of '{}'...", video_path.to_string_lossy().to_string()))
    }

    let output = Command::new(ffprobe).args([
        "-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", video_path_str
        ])
        .output().map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(format!("Error ocurred while retrieving duration of '{}'...", video_path.to_string_lossy().to_string()))
    }

    let stdout = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    let duration = stdout
        .trim()
        .parse::<f64>()
        .map_err(|e| e.to_string())?;
    Ok(duration)
}

fn create_thumbnail(search_path: &String, file_path: &Path, ffmpeg: &PathBuf) -> Result<PathBuf, Box<dyn Error>> {
    let cwd = env::current_dir()?;
    let thumbnails_path = cwd.join("\\thumbnails\\");
    let relative_path = file_path.strip_prefix(search_path)
    .map_err(|e|{
        format!(
            "Error occurred while creating thumbnail for {}... ({})",
            file_path.display(),
            e
        )
    })?;
    let mut video_thumbnail_path = thumbnails_path.join(relative_path);
    video_thumbnail_path.add_extension(".jpg");
    if let Some(video_parent) = video_thumbnail_path.parent() {
        fs::create_dir_all(video_parent)?;
    } 

    let output = Command::new(ffmpeg).args([
        "-i", file_path.to_str().ok_or_else(|| "An unexpected error ocurred while converting video's thumbnail to str...")?,
        "-frames:v", "1",
        "-q:v", "4",
        video_thumbnail_path.to_str().ok_or_else(|| "An unexpected error ocurred while converting video's thumbnail to str...")?
    ])
    .stdout(Stdio::null()).stderr(Stdio::null()).status()?;
    
    if !output.success() {
        return Err(format!("FFMPEG failed while creating thumbnail for '{}'...", file_path.display()).into());
    }

    Ok(video_thumbnail_path)
}

#[tauri::command]
fn list_videos(app: AppHandle, path: String) -> Result<Vec<Video>, String> {
    println!("Processing videos in {}", path.to_string());

    let mut videos: Vec<Video> = Vec::new();
    let ffprobe: PathBuf = get_ffprobe(&app)?;
    let ffmpeg: PathBuf = get_ffmpeg(&app)?;

    for entry in WalkDir::new(&path) {
        let entry = entry.unwrap();
        let file_path: &Path = entry.path();
        if !is_a_video(&file_path) { continue }

        let thumbnail = match create_thumbnail(&path, file_path, &ffmpeg) {
            Ok(p) => p,
            Err(e) => {
                println!("{}", e);
                PathBuf::new()
            }
        };

        videos.push(Video {
            name: get_file_name(&file_path),
            path: file_path.to_string_lossy().to_string(),
            thumbnail_path: thumbnail.to_string_lossy().to_string(),
            duration: get_video_duration(&ffprobe, &file_path).map_err(|e| e.to_string())?
        })
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