import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { convertFileSrc } from "@tauri-apps/api/core";
import "./App.css";

interface Video {
  name: string;
  path: string;
  thumbnail_path: string;
  duration: number;
}

/** Converts seconds (f64) to "H:MM:SS" or "M:SS" */
function formatDuration(secs: number): string {
  const total = Math.floor(secs);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  if (h > 0) {
    return `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  }
  return `${m}:${String(s).padStart(2, "0")}`;
}

/** Strips extension and replaces separators for a cleaner display name */
function cleanName(name: string): string {
  return name.replace(/\.[^/.]+$/, "").replace(/[_\-]+/g, " ");
}

// ── Icons (inline SVG, no extra deps) ──────────────────────────────────────

const IconFolder = () => (
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
    <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
  </svg>
);

const IconRefresh = () => (
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
    <polyline points="23 4 23 10 17 10"/>
    <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
  </svg>
);

const IconPlay = () => (
  <svg viewBox="0 0 24 24" fill="currentColor">
    <polygon points="5,3 19,12 5,21"/>
  </svg>
);

const IconFilm = () => (
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
    <rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18"/>
    <line x1="7" y1="2" x2="7" y2="22"/>
    <line x1="17" y1="2" x2="17" y2="22"/>
    <line x1="2" y1="12" x2="22" y2="12"/>
    <line x1="2" y1="7" x2="7" y2="7"/>
    <line x1="2" y1="17" x2="7" y2="17"/>
    <line x1="17" y1="17" x2="22" y2="17"/>
    <line x1="17" y1="7" x2="22" y2="7"/>
  </svg>
);

const IconAlert = () => (
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
    <circle cx="12" cy="12" r="10"/>
    <line x1="12" y1="8" x2="12" y2="12"/>
    <line x1="12" y1="16" x2="12.01" y2="16"/>
  </svg>
);

// ── VideoCard component ────────────────────────────────────────────────────

interface VideoCardProps {
  video: Video;
}

function VideoCard({ video }: VideoCardProps) {
  const [thumbError, setThumbError] = useState(false);
  const thumbSrc = convertFileSrc(video.thumbnail_path);

  return (
    <article className="video-card" title={video.name}>
      <div className="video-card__thumb-wrap">
        {!thumbError ? (
          <img
            className="video-card__thumb"
            src={thumbSrc}
            alt={cleanName(video.name)}
            loading="lazy"
            onError={() => setThumbError(true)}
          />
        ) : (
          <div className="video-card__thumb--missing">
            <IconFilm />
          </div>
        )}

        {/* Play overlay */}
        <div className="video-card__play-overlay">
          <div className="video-card__play-icon">
            <IconPlay />
          </div>
        </div>

        {/* Duration badge */}
        <span className="video-card__duration">
          {formatDuration(video.duration)}
        </span>
      </div>

      <div className="video-card__info">
        <p className="video-card__name">{cleanName(video.name)}</p>
        <p className="video-card__meta">{video.name}</p>
      </div>
    </article>
  );
}

// ── App ────────────────────────────────────────────────────────────────────

function App() {
  const [videoList, setVideoList] = useState<Video[]>([]);
  const [searchFolder, setSearchFolder] = useState<string>(".");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const listVideos = async () => {
    setError(null);
    setLoading(true);
    try {
      const videosFound: Video[] = await invoke("list_videos", { path: searchFolder });
      setVideoList(videosFound);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const openFolderSelector = async () => {
    const selected = await open({ directory: true, multiple: false });
    if (selected != null) {
      setSearchFolder(selected as string);
    }
  };

  return (
    <div className="app">

      {/* ── Header ── */}
      <header className="header">
        <h1 className="header__logo">
          OC<span>U</span>LI
        </h1>

        <span className="header__folder-path" title={searchFolder}>
          {searchFolder}
        </span>

        <div className="header__controls">
          <button className="btn btn--ghost" onClick={openFolderSelector}>
            <IconFolder />
            Carpeta
          </button>
          <button className="btn btn--primary" onClick={listVideos} disabled={loading}>
            <IconRefresh />
            {loading ? "Cargando…" : "Escanear"}
          </button>
        </div>
      </header>

      {/* ── Error banner ── */}
      {error && (
        <div className="error-banner">
          <IconAlert />
          {error}
        </div>
      )}

      {/* ── Status bar (only when there are results) ── */}
      {videoList.length > 0 && (
        <div className="status-bar">
          <span className="status-bar__count">
            <strong>{videoList.length}</strong> video{videoList.length !== 1 ? "s" : ""} encontrado{videoList.length !== 1 ? "s" : ""}
          </span>
          <span className="status-bar__count" style={{ fontStyle: "italic" }}>
            {searchFolder}
          </span>
        </div>
      )}

      {/* ── Main content ── */}
      <main className="main">
        {videoList.length === 0 && !loading ? (
          <div className="empty-state">
            <div className="empty-state__icon">
              <IconFilm />
            </div>
            <p className="empty-state__title">Sin videos</p>
            <p className="empty-state__sub">
              Seleccioná una carpeta y presioná <em>Escanear</em> para encontrar tus videos.
            </p>
          </div>
        ) : (
          <div className="video-grid">
            {videoList.map((video, index) => (
              <VideoCard key={`${video.path}-${index}`} video={video} />
            ))}
          </div>
        )}
      </main>
    </div>
  );
}

export default App;