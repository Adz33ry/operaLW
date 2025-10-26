# Opera Live Wallpaper (operaLW)

A small macOS desktop utility that packages a video into a wallpaper theme for Opera/Opera GX. Drop a video, optionally trim a segment, adaptively crop to 16:9 if needed, transcode to VP9, and install it as a browser theme.

## Features
- Drag & drop a video or choose via “Add” (`.mp4`, `.mov`, `.webm`, `.gif`).
- Auto‑detect metadata (duration, resolution, FPS, VFR).
- Optional time‑based trimming (Start/End).
- Adaptive crop to 16:9 or “No crop”.
- Choose output height: 1080p/1440p/2160p (one option).
- Export and install the theme into Opera GX’s themes directory (restart browser afterwards).

## Requirements
- macOS 12.0+.
- FFmpeg (with `ffprobe`) installed on the system.
  - Recommended via Homebrew: `brew install ffmpeg`.

## Install & Run
- Direct install (recommended): download the latest `.dmg` from GitHub Releases and drag the app into Applications.
  - Latest release: https://github.com/Adz33ry/operaLW/releases/latest
  - Full list: https://github.com/Adz33ry/operaLW/releases

- Build locally: generate a `.dmg` and install the app
  - `pnpm install`
  - `pnpm tauri:build`
  - Install the generated `Opera Live Wallpaper.dmg` and launch the app from Applications.

- Development (dev):
  - `pnpm install`
  - `pnpm tauri:dev`

## End‑User Install (macOS)
These steps are for non‑technical users who just want to install and run the app from GitHub.

- Download the `.dmg` from the latest Release.
- Double‑click the `.dmg` to mount it.
- In the window that opens, drag `Opera Live Wallpaper.app` into `Applications`.
- Eject the mounted disk (right‑click → Eject), then open `Applications` and launch the app.

First launch on macOS
- If the app is signed and notarized, it opens normally.
- If you see a security prompt (unidentified developer or “can’t be opened”):
  - Right‑click on `Opera Live Wallpaper.app` → Open → Open (this is a one‑time action per app).
  - Or go to System Settings → Privacy & Security → scroll to the bottom → “Open Anyway”.
- If you see “is damaged and can’t be opened” after downloading from a browser, it’s Gatekeeper quarantine. The two no‑terminal ways above usually work. If needed and you’re comfortable with Terminal, you can remove the quarantine attribute:
  - `xattr -dr com.apple.quarantine "/Applications/Opera Live Wallpaper.app"`

Notes
- Always run the app from `Applications` (not directly from inside the `.dmg`).
- On first use, Opera GX may require a browser restart to pick up the newly installed theme.

## Usage
1. Launch the app and drop a video file into the window, or click “Add”.
2. Wait for the metadata to appear (duration, size, FPS).
3. If needed, set trim times (Start/End) or click “Don't Trim”.
4. Choose a target height (1080p / 1440p / 2160p) or enable “No crop”.
5. Click “Install”. The theme ZIP will be saved into the Opera GX themes folder.
6. Restart Opera/Opera GX and select the new theme in the browser settings.

Where the theme is saved (macOS):
- `~/Library/Application Support/com.operasoftware.OperaGX/themes/` — the ZIP theme file.

Temporary files:
- `~/Library/Caches/opera-lw/` — intermediate encoding files are cleaned up after export.

## FFmpeg in the packaged app
When launched from Finder, macOS GUI apps may not inherit your shell `PATH`. The app will try common locations for `ffmpeg`/`ffprobe` (Homebrew: `/opt/homebrew/bin`, Intel: `/usr/local/bin`, system: `/usr/bin`).

If FFmpeg is installed in a non‑standard location, set environment variables with full paths to the binaries:
- `OPERALW_FFMPEG=/full/path/to/ffmpeg`
- `OPERALW_FFPROBE=/full/path/to/ffprobe`

Quick checks (in Terminal):
- `/opt/homebrew/bin/ffmpeg -version` (Apple Silicon)
- `/usr/local/bin/ffmpeg -version` (Intel)

Run the installed app from Terminal (for diagnostics):
- `/Applications/Opera Live Wallpaper.app/Contents/MacOS/opera-lw`

## Frontend build
- `pnpm build` — builds static assets to `dist/`. Tauri also runs this via `beforeBuildCommand`.

## Technical details
- Framework: Tauri v2 (Rust backend + Vite/TypeScript frontend).
- Video codec: VP9 (`libvpx-vp9`, `-crf 30`, CFR up to 30 FPS). Poster: first frame JPG.
- Theme file format: ZIP with `persona.ini`, `background.webm`, `first_frame_start_page.jpg` at archive root.

## Support
If this tool helps you, consider a Ko‑fi: https://ko-fi.com/K3K21NBUDO

## License
FFmpeg licensing: see `LICENSES/ffmpeg.txt`. The application’s own license is defined by the repository owner.
