use serde::Deserialize;
use serde::Serialize;
use std::process::Command as StdCommand;
use std::{fs, io::Write, path::Path, path::PathBuf};
use tauri::command;

#[derive(Serialize, Deserialize)]
pub struct VideoMeta {
    pub path: String,
    pub duration: f64,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub vfr: bool,
    pub codec: String,
}

#[derive(Deserialize)]
struct FfprobeFormat {
    duration: Option<String>,
}

#[derive(Deserialize)]
struct FfprobeStream {
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    avg_frame_rate: Option<String>,
    r_frame_rate: Option<String>,
    nb_frames: Option<String>,
}

#[derive(Deserialize)]
struct FfprobeJson {
    streams: Option<Vec<FfprobeStream>>,
    format: Option<FfprobeFormat>,
}

fn parse_fraction(fr: &str) -> Option<f64> {
    let parts: Vec<&str> = fr.split('/').collect();
    if parts.len() == 2 {
        let num: f64 = parts[0].parse().ok()?;
        let den: f64 = parts[1].parse().ok()?;
        if den == 0.0 { return None; }
        Some(num / den)
    } else {
        fr.parse().ok()
    }
}

#[command]
pub async fn ping() -> Result<&'static str, String> { Ok("ok") }

#[command]
pub async fn probe_video(path: String) -> Result<VideoMeta, String> {
    // call `ffprobe` from PATH; Stage 1 will switch to sidecar
    let args = [
        "-v",
        "error",
        "-select_streams",
        "v:0",
        "-show_entries",
        "stream=codec_name,width,height,avg_frame_rate,r_frame_rate,nb_frames",
        "-show_entries",
        "format=duration",
        "-of",
        "json",
        &path,
    ];

    let out = StdCommand::new("ffprobe")
        .args(&args)
        .output()
        .map_err(|e| format!("failed to spawn ffprobe: {}", e))?;

    if !out.status.success() {
        return Err(format!(
            "ffprobe exited with {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        ));
    }

    let parsed: FfprobeJson = serde_json::from_slice(&out.stdout)
        .map_err(|e| format!("ffprobe parse error: {}", e))?;

    let stream = parsed
        .streams
        .as_ref()
        .and_then(|v| v.get(0))
        .ok_or_else(|| "Unsupported format (no video stream)".to_string())?;

    let duration = parsed
        .format
        .as_ref()
        .and_then(|f| f.duration.as_ref())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    let avg = stream
        .avg_frame_rate
        .as_deref()
        .and_then(parse_fraction)
        .unwrap_or(0.0);
    let r = stream
        .r_frame_rate
        .as_deref()
        .and_then(parse_fraction)
        .unwrap_or(avg);
    let nb_frames_missing = stream.nb_frames.is_none();
    let vfr = (avg - r).abs() > f64::EPSILON || nb_frames_missing;

    Ok(VideoMeta {
        path,
        duration,
        width: stream.width.unwrap_or(0),
        height: stream.height.unwrap_or(0),
        fps: avg,
        vfr,
        codec: stream.codec_name.clone().unwrap_or_default(),
    })
}

#[derive(Deserialize)]
pub struct ExportRequest {
    pub path: String,
    pub start: f64,
    pub end: f64,
    pub sizes: Vec<u32>,
    pub name: String,
}

fn home_dir() -> Result<PathBuf, String> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| "HOME not set".to_string())
}

fn run(cmd: &mut StdCommand) -> Result<(), String> {
    let out = cmd.status().map_err(|e| format!("spawn failed: {}", e))?;
    if !out.success() {
        return Err(format!(
            "command failed: {:?}",
            out.code()
        ));
    }
    Ok(())
}

#[command]
pub async fn export_package(req: ExportRequest) -> Result<String, String> {
    // Ensure ffmpeg/ffprobe available
    run(StdCommand::new("ffprobe").arg("-version"))?;
    run(StdCommand::new("ffmpeg").arg("-version"))?;

    // Probe input to decide FPS policy
    let meta = probe_video(req.path.clone()).await?;
    let input_fps = meta.fps;
    let mut target_fps = input_fps.round() as u32;
    if target_fps > 30 { target_fps = 30; }
    if target_fps == 0 { target_fps = 30; }

    let start = req.start.max(0.0);
    let mut end = req.end.max(start);
    let source_duration = meta.duration;
    if end == 0.0 || end > source_duration { end = source_duration; }
    let seg_duration = (end - start).max(0.0);

    // Prepare temp folder
    let base = home_dir()?.join("Library").join("Caches").join("opera-lw");
    fs::create_dir_all(&base).map_err(|e| format!("mkdir base: {}", e))?;
    let id = uuid::Uuid::new_v4().to_string();
    let tmp = base.join(&id);
    fs::create_dir_all(&tmp).map_err(|e| format!("mkdir tmp: {}", e))?;

    let mut generated: Vec<(PathBuf, String)> = Vec::new();
    let mut last_err: Option<String> = None;

    // Export a single background.webm (prefer highest requested height)
    let chosen_h = if req.sizes.contains(&2160) {
        Some(2160)
    } else if req.sizes.contains(&1440) {
        Some(1440)
    } else if req.sizes.contains(&1080) {
        Some(1080)
    } else {
        None
    };

    let out_video = tmp.join("background.webm");
    let mut args: Vec<String> = vec![
        "-y".into(),
        "-ss".into(), format!("{}", start),
        "-to".into(), format!("{}", end),
        "-i".into(), req.path.clone(),
    ];
    if let Some(h) = chosen_h {
        args.extend(["-vf".into(), format!("scale=-2:{},crop=iw:trunc(iw*9/16)", h)]);
    }
    args.extend([
        "-c:v".into(), "libvpx-vp9".into(),
        "-b:v".into(), "0".into(),
        "-crf".into(), "30".into(),
        "-pix_fmt".into(), "yuv420p".into(),
        "-row-mt".into(), "1".into(),
        "-tile-columns".into(), "2".into(),
        "-threads".into(), "0".into(),
        "-an".into(),
        "-r".into(), target_fps.to_string(),
        "-vsync".into(), "cfr".into(),
        out_video.to_string_lossy().into(),
    ]);
    let status = StdCommand::new("ffmpeg").args(args.iter().map(|s| s.as_str())).output();
    match status {
        Ok(o) if o.status.success() => {
            generated.push((out_video.clone(), "background.webm".into()));
        }
        Ok(o) => last_err = Some(format!("ffmpeg video failed: {}", String::from_utf8_lossy(&o.stderr))),
        Err(e) => last_err = Some(format!("spawn ffmpeg video: {}", e)),
    }
    if generated.is_empty() {
        let _ = fs::remove_dir_all(&tmp);
        return Err(last_err.unwrap_or_else(|| "no outputs generated".into()));
    }

    // Poster (single-frame JPEG) exact file name
    let poster = tmp.join("first_frame_start_page.jpg");
    crate::ffmpeg::extract_poster(Path::new(&req.path), poster.as_path())
        .map_err(|e| format!("{}", e))?;
    generated.push((poster.clone(), "first_frame_start_page.jpg".into()));

    // No metadata.json in this packaging

    // Create ZIP directly into Opera GX themes folder
    let app_support = home_dir()?
        .join("Library")
        .join("Application Support")
        .join("com.operasoftware.OperaGX");
    let themes_dir = app_support.join("themes");
    fs::create_dir_all(&themes_dir).map_err(|e| format!("mkdir themes: {}", e))?;

    // Sanitize filename and ensure uniqueness
    fn sanitize(name: &str) -> String {
        let base: String = name
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        if base.is_empty() { "wallpaper".into() } else { base }
    }
    let zip_out = themes_dir.join(format!("{}.zip", sanitize(&req.name)));
    let zip_file = fs::File::create(&zip_out).map_err(|e| format!("zip create: {}", e))?;
    let mut zip = zip::ZipWriter::new(zip_file);
    let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    // Write persona.ini and assets at root (no subfolders)
    let persona_contents = format!(
        "[Info]\nName={}\nVersion=1\n\n[Start Page]\nbackground = background.webm\nfirst frame image = first_frame_start_page.jpg\n",
        req.name
    );
    zip.start_file("persona.ini", options).map_err(|e| format!("zip start persona.ini: {}", e))?;
    zip.write_all(persona_contents.as_bytes()).map_err(|e| format!("zip write persona.ini: {}", e))?;

    let bg_bytes = fs::read(&out_video).map_err(|e| format!("read background.webm: {}", e))?;
    zip.start_file("background.webm", options).map_err(|e| format!("zip start background.webm: {}", e))?;
    zip.write_all(&bg_bytes).map_err(|e| format!("zip write background.webm: {}", e))?;

    let poster_bytes = fs::read(&poster).map_err(|e| format!("read first_frame_start_page.jpg: {}", e))?;
    zip.start_file("first_frame_start_page.jpg", options).map_err(|e| format!("zip start first_frame_start_page.jpg: {}", e))?;
    zip.write_all(&poster_bytes).map_err(|e| format!("zip write first_frame_start_page.jpg: {}", e))?;
    zip.finish().map_err(|e| format!("zip finish: {}", e))?;

    // Do NOT open Finder or Opera. User will pick the wallpaper inside the browser UI.

    // Cleanup temp
    let _ = fs::remove_dir_all(&tmp);

    Ok(zip_out.to_string_lossy().into_owned())
}
