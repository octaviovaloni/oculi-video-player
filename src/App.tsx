import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const func = async () => {
    invoke("list_videos", {path: "D:/NVIDIA VIDEOS"})
  }

  return (
    <div>
      <h1>Oculi (Pre Alpha)</h1>
      <button onClick={func}>Invoke "func"</button>
    </div>
  );
}

export default App;
