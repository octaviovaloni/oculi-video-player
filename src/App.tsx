import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const func = async () => {
    invoke("func", {})
  }

  return (
    <div>
      <button onClick={func}>Invoke "func"</button>
    </div>
  );
}

export default App;
