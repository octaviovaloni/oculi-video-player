// scripts/download-ffmpeg.mjs
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import https from "node:https";
import { spawnSync } from "node:child_process";

const isWindows = process.platform === "win32";
if (!isWindows) {
  console.log("[ffmpeg] Skipping: script currently configured for Windows only.");
  process.exit(0);
}

const ROOT = process.cwd();
const BIN_DIR = path.join(ROOT, "src-tauri", "bin");
const MARKER = path.join(BIN_DIR, ".ffmpeg-version");

const FFMPEG_ZIP_URL = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-git-essentials.7z";
const VERSION_TAG = "gyan-ffmpeg-git-essentials";

function ensureDir(dir) {
  fs.mkdirSync(dir, { recursive: true });
}

function fileExists(p) {
  try { fs.accessSync(p); return true; } catch { return false; }
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    https.get(url, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        file.close();
        fs.unlinkSync(dest);
        return resolve(download(res.headers.location, dest));
      }

      if (res.statusCode !== 200) {
        file.close();
        return reject(new Error(`HTTP ${res.statusCode} downloading ${url}`));
      }

      res.pipe(file);
      file.on("finish", () => file.close(resolve));
    }).on("error", (err) => {
      try { fs.unlinkSync(dest); } catch {}
      reject(err);
    });
  });
}

function requirePowerShell() {
  const r = spawnSync("powershell", ["-NoProfile", "-Command", "$PSVersionTable.PSVersion.ToString()"], { encoding: "utf8" });
  if (r.status !== 0) {
    throw new Error("PowerShell not available. Unable to download FFMPEG/FFPROBE without powershell");
  }
}

function extractZip(zipPath, outDir) {
  requirePowerShell();
  const cmd = `Expand-Archive -LiteralPath "${zipPath}" -DestinationPath "${outDir}" -Force`;
  const r = spawnSync("powershell", ["-NoProfile", "-Command", cmd], { stdio: "inherit" });
  if (r.status !== 0) throw new Error("Expand-Archive failed...");
}

function findExe(extractDir, exeName) {
  const stack = [extractDir];
  while (stack.length) {
    const dir = stack.pop();
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    for (const e of entries) {
      const p = path.join(dir, e.name);
      if (e.isDirectory()) stack.push(p);
      else if (e.isFile() && e.name.toLowerCase() === exeName) return p;
    }
  }
  return null;
}

async function main() {
  ensureDir(BIN_DIR);

  const ffmpegOut = path.join(BIN_DIR, "ffmpeg.exe");
  const ffprobeOut = path.join(BIN_DIR, "ffprobe.exe");

  if (fileExists(ffmpegOut) && fileExists(ffprobeOut) && fileExists(MARKER)) {
    const v = fs.readFileSync(MARKER, "utf8").trim();
    if (v === VERSION_TAG) {
      console.log("[ffmpeg] Already present. Skipping download.");
      return;
    }
  }

  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "ffmpeg-"));
  const zipPath = path.join(tmpDir, "ffmpeg.zip");
  const extractDir = path.join(tmpDir, "extract");

  console.log("[ffmpeg] Downloading...");
  await download(FFMPEG_ZIP_URL, zipPath);

  console.log("[ffmpeg] Extracting...");
  extractZip(zipPath, extractDir);

  const ffmpegSrc = findExe(extractDir, "ffmpeg.exe");
  const ffprobeSrc = findExe(extractDir, "ffprobe.exe");

  if (!ffmpegSrc || !ffprobeSrc) {
    throw new Error("Couldn't find ffmpeg.exe or ffprobe.exe inside the downloaded .7z");
  }

  console.log("[ffmpeg] Installing to src-tauri/bin...");
  fs.copyFileSync(ffmpegSrc, ffmpegOut);
  fs.copyFileSync(ffprobeSrc, ffprobeOut);

  fs.writeFileSync(MARKER, VERSION_TAG, "utf8");

  console.log("[ffmpeg] Done.");
}

main().catch((err) => {
  console.error("[ffmpeg] ERROR:", err.message);
  process.exit(1);
});