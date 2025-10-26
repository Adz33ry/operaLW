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
  - Releases: https://github.com/Adz33ry/operaLW/releases

- Build locally: generate a `.dmg` and install the app
  - `pnpm install`
  - `pnpm tauri:build`
  - Install the generated `Opera Live Wallpaper.dmg` and launch the app from Applications.

- Development (dev):
  - `pnpm install`
  - `pnpm tauri:dev`

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

## Known limitations
- Currently exports to the Opera GX themes folder; additional Opera directories may be supported later.
- Encoding long videos can take significant time.

## License
FFmpeg licensing: see `LICENSES/ffmpeg.txt`. The application’s own license is defined by the repository owner.
