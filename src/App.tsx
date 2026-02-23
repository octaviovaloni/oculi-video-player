import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

type Video = {
    name: string
    path: String
    miniature_path: String
    duration: number
}

function App() {
  const [video_list, set_video_list] = useState<Video[]>([])
  const [search_folder, set_search_folder] = useState<string>(".")

  const list_videos = async () => {
    try {
      const videos_found: Video[] = await invoke("list_videos", {path: search_folder});
      set_video_list(videos_found)
    } catch (e) {
      console.error("Error desde Rust:", e);
      alert("Error desde Rust:\n" + String(e));
    }
  }
  const open_folder_selector = async () => {
    const selected = await open({directory: true, multiple: false});
    if (selected != null) {
      set_search_folder(selected)
    } else {
      alert("No folder selected. Search folder: " + search_folder)
    }
  }

  return (
    <div>
      <h1>Oculi (Pre Alpha)</h1>
      <button onClick={list_videos}>List Videos</button>
      <button onClick={open_folder_selector}>Select Folder</button>
      <p>Search Folder: {search_folder}</p>
      <ul>
        {video_list.map((video: Video, index) => (
          <li key={index}>{video.name} | {video.duration} | {video.path}</li>
        ))}
      </ul>
    </div>
  );
}

export default App;
