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

fn app_data_dir(app: &AppHandle) -> PathBuf {
    app.path().resource_dir().unwrap().join("Oculi")
}

fn thumbnails_dir(app: &AppHandle) -> Result<PathBuf, Box<dyn Error>> {
    let thumbnail_directory = app_data_dir(app).join("thumbnails");
    fs::create_dir_all(&thumbnail_directory)?;
    Ok(thumbnail_directory)
}

fn get_ffprobe(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path().resolve("bin/ffprobe.exe", BaseDirectory::Resource)
    .map_err(|e| {
        format!("Error ocurred while accessing ffprobe.exe... ({})", e)
    })
}

fn get_ffmpeg(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path().resolve("bin/ffmpeg.exe", BaseDirectory::Resource)
    .map_err(|e| {
        format!("Error ocurred while accessing ffmpeg.exe... ({})", e)
    })
}

fn is_a_video(path: &Path) -> bool {
    const VIDEO_EXTENSIONS: &[&str] = &[
        "mp4", "mkv", "avi", "mov", "webm", "flv", "wmv", "mpeg",
    ];

    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| VIDEO_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn get_file_name(path: &Path) -> String {
    if let Some(name) = path.file_name() { return name.to_string_lossy().to_string() } else { return "NAME_ERROR".to_string() }
}

fn get_video_duration(ffprobe: &PathBuf, video_path: &Path) -> Result<f64, String> {
    let path_display = video_path.to_string_lossy();
    let duration_err = format!("Error retrieving duration of '{path_display}'");

    let video_path_str = video_path
        .to_str()
        .ok_or_else(|| &duration_err)?;

    let output = Command::new(ffprobe)
        .args([
            "-v",            "error",
            "-show_entries", "format=duration",
            "-of",           "default=noprint_wrappers=1:nokey=1",
            video_path_str,
        ])
        .output()
        .map_err(|e| format!("{} ({})", duration_err, e))?;

    if !output.status.success() {
        return Err(duration_err);
    }

    String::from_utf8(output.stdout)
        .map_err(|e| e.to_string())?
        .trim()
        .parse::<f64>()
        .map_err(|e| e.to_string())
}

fn create_thumbnail(app: &AppHandle, search_path: &String, file_path: &Path, ffmpeg: &PathBuf) -> Result<PathBuf, Box<dyn Error>> {
    let thumbnails_path = thumbnails_dir(app)?;
    let relative_path: &Path = file_path.strip_prefix(search_path)?;
    let mut video_thumbnail_path: PathBuf = thumbnails_path.join(relative_path);
    video_thumbnail_path.add_extension("jpg");

    if let Some(video_parent) = video_thumbnail_path.parent() {
        fs::create_dir_all(video_parent)?;
    }

    if video_thumbnail_path.exists() {
        fs::remove_file(&video_thumbnail_path)?;
    }

    let output = Command::new(ffmpeg)
        .args([
            "-i", file_path.to_str().ok_or("An unexpected error ocurred while converting video's thumbnail to str...")?,
            "-frames:v", "1",
            "-q:v", "4",
            video_thumbnail_path.to_str().ok_or("An unexpected error ocurred while converting video's thumbnail to str...")?,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    
    if !output.success() {
        return Err(format!("FFMPEG failed while creating thumbnail for '{}'...", file_path.display()).into());
    }

    Ok(video_thumbnail_path)
}

#[tauri::command]
fn list_videos(app: AppHandle, path: String) -> Result<Vec<Video>, String> {
    println!("Processing videos in '{path}'...");

    let ffprobe = get_ffprobe(&app)?;
    let ffmpeg  = get_ffmpeg(&app)?;

    let videos: Vec<Video> = WalkDir::new(&path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path().to_path_buf())
        .filter(|file_path| is_a_video(file_path))
        .map(|file_path| {
            let thumbnail = create_thumbnail(&app, &path, &file_path, &ffmpeg)
                .unwrap_or_else(|e| {
                    println!("Error creating thumbnail for '{}'... ({e})", file_path.display());
                    PathBuf::new()
                });

            Video {
                name:           get_file_name(&file_path),
                path:           file_path.to_string_lossy().to_string(),
                thumbnail_path: thumbnail.to_string_lossy().to_string(),
                duration:       get_video_duration(&ffprobe, &file_path).unwrap_or(0.0),
            }
        })
        .collect();

    println!("Found {} videos...", videos.len());

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