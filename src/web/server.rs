use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use clap::Parser;
use desmos_midi::audio::{analyze_harmonics, read_wav_file, AnalysisConfig, AudioError};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    net::SocketAddr,
    path::{Path as StdPath, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{fs, io::AsyncWriteExt, net::TcpListener, task, time};
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// CLI Arguments
#[derive(Parser)]
#[command(author, version, about = "Desmos MIDI Web UI Server")]
struct Args {
    /// Port to run the server on
    #[arg(short, long, default_value_t = 8573)]
    port: u16,

    /// Don't open browser automatically
    #[arg(long, default_value_t = false)]
    no_open_browser: bool,
}

// Server configuration
#[derive(Deserialize)]
struct Config {
    server: ServerConfig,
}

#[derive(Deserialize)]
struct ServerConfig {
    file_expiration_minutes: u64,
    file_refresh_threshold_minutes: u64,
    max_file_size_mb: u64,
    limits: AnalysisLimits,
}

#[derive(Deserialize)]
struct AnalysisLimits {
    min_samples: usize,
    max_samples: usize,
    min_start_time: f32,
    max_start_time: f32,
    min_base_freq: f32,
    max_base_freq: f32,
    min_harmonics: usize,
    max_harmonics: usize,
    min_boost: f32,
    max_boost: f32,
}

// App state
#[derive(Clone)]
struct AppState {
    temp_dir: PathBuf,
    soundfont_dir: PathBuf,
    file_expirations: Arc<Mutex<HashMap<String, Instant>>>,
    config: Arc<ServerConfig>,
}

// Response for MIDI info
#[derive(Serialize)]
struct MidiInfoResponse {
    channels: Vec<ChannelInfo>,
}

#[derive(Serialize)]
struct ChannelInfo {
    id: u8,
    instrument: String,
    is_drum: bool,
}

// Response for conversion
#[derive(Serialize)]
struct ConversionResponse {
    formula: String,
}

// Request for conversion with soundfonts
#[derive(Deserialize)]
struct ConversionRequest {
    filename: String,
    soundfonts: Vec<String>,
}

// Request for refreshing file expiration
#[derive(Deserialize)]
struct RefreshFileRequest {
    filename: String,
}

// Response for harmonic analysis
#[derive(Serialize)]
struct HarmonicResponse {
    harmonics: Vec<f32>,
}

// Query parameters for harmonic analysis
#[derive(Deserialize)]
struct HarmonicParams {
    samples: Option<usize>,
    #[serde(rename = "startTime")]
    start_time: Option<f32>,
    #[serde(rename = "baseFreq")]
    base_freq: Option<f32>,
    harmonics: Option<usize>,
    boost: Option<f32>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "desmos_midi_web=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse command line arguments
    let args = Args::parse();

    // Load configuration
    let config = load_config().unwrap_or_else(|e| {
        tracing::warn!("Failed to load config.toml: {}. Using default values.", e);
        Config {
            server: ServerConfig {
                file_expiration_minutes: 10,
                file_refresh_threshold_minutes: 5,
                max_file_size_mb: 80,
                limits: AnalysisLimits {
                    min_samples: 64,
                    max_samples: 65536,
                    min_start_time: 0.0,
                    max_start_time: 300.0,
                    min_base_freq: 1.0,
                    max_base_freq: 20000.0,
                    min_harmonics: 1,
                    max_harmonics: 128,
                    min_boost: 0.5,
                    max_boost: 2.0,
                },
            },
        }
    });

    // Create temp directory if it doesn't exist
    let temp_dir = PathBuf::from("temp");
    if !temp_dir.exists() {
        fs::create_dir_all(&temp_dir).await.unwrap();
    } else {
        // Clean up existing files on startup
        clean_temp_directory(&temp_dir).await;
    }

    // Get soundfont directory
    let soundfont_dir = PathBuf::from("soundfonts");
    if !soundfont_dir.exists() {
        fs::create_dir_all(&soundfont_dir).await.unwrap();
    }

    // Create app state with file expiration tracking
    let file_expirations = Arc::new(Mutex::new(HashMap::new()));
    let state = Arc::new(AppState {
        temp_dir,
        soundfont_dir,
        file_expirations,
        config: Arc::new(config.server),
    });

    // Create static directory and subdirectories if they don't exist
    let static_dir = PathBuf::from("src/web/static");
    if !static_dir.exists() {
        fs::create_dir_all(&static_dir).await.unwrap();
    }

    // Create js directory
    let js_dir = static_dir.join("js");
    if !js_dir.exists() {
        fs::create_dir_all(&js_dir).await.unwrap();
    }

    // Create css directory
    let css_dir = static_dir.join("css");
    if !css_dir.exists() {
        fs::create_dir_all(&css_dir).await.unwrap();
    }

    // Create modules directory inside js
    let modules_dir = js_dir.join("modules");
    if !modules_dir.exists() {
        fs::create_dir_all(&modules_dir).await.unwrap();
    }

    // Start background task for file cleanup
    let cleanup_state = state.clone();
    task::spawn(async move {
        run_file_cleanup(cleanup_state).await;
    });

    // Create router
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/wav-to-soundfont", get(wav_to_soundfont_handler))
        .route("/upload", post(upload_handler))
        .route("/midi-info/{filename}", get(midi_info_handler))
        .route("/convert", post(convert_handler))
        .route("/soundfonts", get(list_soundfonts_handler))
        .route("/refresh-file", post(refresh_file_handler))
        .route("/getfile/{filename}", get(get_file_handler))
        .route("/save-soundfont/{filename}", post(save_soundfont_handler))
        .route("/harmonic-info/{filename}", get(harmonic_info_handler))
        .nest_service("/static", ServeDir::new(StdPath::new("src/web/static")))
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    tracing::info!("Listening on http://localhost:{}", args.port);

    // Open browser if requested
    if !args.no_open_browser {
        open_browser(args.port);
    }

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Open browser based on the operating system
fn open_browser(port: u16) {
    let url = format!("http://localhost:{}", port);

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let _ = Command::new("cmd").args(["/C", "start", &url]).spawn();
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let _ = Command::new("open").arg(&url).spawn();
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        // Try different commands to open the browser
        let commands = ["xdg-open", "gnome-open", "sensible-browser"];
        for cmd in commands {
            if Command::new(cmd).arg(&url).spawn().is_ok() {
                break;
            }
        }
    }

    // For other platforms, just log the URL
    tracing::info!("Server started. Please open {} in your browser", url);
}

// Clean up all files in the temp directory
async fn clean_temp_directory(temp_dir: &PathBuf) {
    tracing::info!("Cleaning up temporary files on startup");

    match fs::read_dir(temp_dir).await {
        Ok(mut entries) => {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Err(e) = fs::remove_file(entry.path()).await {
                    tracing::warn!("Failed to remove file {}: {}", entry.path().display(), e);
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to read temp directory: {}", e);
        }
    }
}

// Background task to periodically check for and remove expired files
async fn run_file_cleanup(state: Arc<AppState>) {
    let check_interval = Duration::from_secs(60); // Check every minute

    loop {
        time::sleep(check_interval).await;

        let now = Instant::now();
        let mut expired_files = Vec::new();

        // Find expired files
        {
            let mut expirations = state.file_expirations.lock().unwrap();
            let expiration_duration =
                Duration::from_secs(state.config.file_expiration_minutes * 60);

            expirations.retain(|filename, expiration_time| {
                let is_expired = now.duration_since(*expiration_time) >= expiration_duration;
                if is_expired {
                    expired_files.push(filename.clone());
                    false
                } else {
                    true
                }
            });
        }

        // Remove expired files
        for filename in expired_files {
            let file_path = state.temp_dir.join(&filename);
            if let Err(e) = fs::remove_file(&file_path).await {
                tracing::warn!(
                    "Failed to remove expired file {}: {}",
                    file_path.display(),
                    e
                );
            } else {
                tracing::info!("Removed expired file: {}", filename);
            }
        }
    }
}

// Handler for the index page
async fn index_handler() -> impl IntoResponse {
    let html = include_str!("static/index.html");
    Html(html)
}

// Handler for the wav_to_soundfont page
async fn wav_to_soundfont_handler() -> impl IntoResponse {
    let html = include_str!("static/wav_to_soundfont.html");
    Html(html)
}

// Handler for uploading MIDI files
async fn upload_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut file_path = None;
    let mut original_filename = String::new();

    // Process the multipart form
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Failed to process form: {}", e),
        )
    })? {
        let name = field.name().unwrap_or("").to_string();
        if name == "midi_file" {
            let file_name = field
                .file_name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown.file".to_string());

            // Use the original filename directly
            original_filename = file_name;

            // Get the file data
            let data = field.bytes().await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to read file: {}", e),
                )
            })?;

            // Check file size
            let max_size = state.config.max_file_size_mb * 1024 * 1024; // Convert MB to bytes
            if data.len() > max_size as usize {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!(
                        "File too large. Maximum size is {} MB",
                        state.config.max_file_size_mb
                    ),
                ));
            }

            // Create the file path
            let path = state.temp_dir.join(&original_filename);
            file_path = Some(path.clone());

            // Write the file
            let mut file = fs::File::create(&path).await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to create file: {}", e),
                )
            })?;
            file.write_all(&data).await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to write file: {}", e),
                )
            })?;

            // Set expiration time
            {
                let mut expirations = state.file_expirations.lock().unwrap();
                expirations.insert(original_filename.clone(), Instant::now());
            }
        }
    }

    // Return the filename with config values
    match file_path {
        Some(_) => Ok(Json(serde_json::json!({
            "filename": original_filename,
            "expires_in_minutes": state.config.file_expiration_minutes,
            "refresh_threshold_minutes": state.config.file_refresh_threshold_minutes
        }))),
        None => Err((
            StatusCode::BAD_REQUEST,
            "No MIDI file was uploaded".to_string(),
        )),
    }
}

// Handler for getting MIDI file information
async fn midi_info_handler(
    State(state): State<Arc<AppState>>,
    Path(filename): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Check if the file exists
    let file_path = state.temp_dir.join(&filename);
    if !file_path.exists() {
        return Err((StatusCode::NOT_FOUND, "MIDI file not found".to_string()));
    }

    // Refresh the file expiration
    {
        let mut expirations = state.file_expirations.lock().unwrap();
        expirations.insert(filename, Instant::now());
    }

    // Create MIDI processor
    let processor = ::desmos_midi::midi::MidiProcessor::with_soundfont_dir(
        state.soundfont_dir.to_str().unwrap(),
    );

    // Process the MIDI file
    let song = processor
        .process_info(file_path.to_str().unwrap())
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to process MIDI file: {}", e),
            )
        })?;

    // Convert to response format
    let channels = song
        .channels
        .iter()
        .map(|ch| ChannelInfo {
            id: ch.id + 1, // MIDI channels are 1-based in display
            instrument: ::desmos_midi::midi::get_instrument_name(ch.instrument, ch.is_drum)
                .to_string(),
            is_drum: ch.is_drum,
        })
        .collect();

    Ok(Json(MidiInfoResponse { channels }))
}

// Handler for converting MIDI files
async fn convert_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ConversionRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Check if the file exists
    let file_path = state.temp_dir.join(&request.filename);
    if !file_path.exists() {
        return Err((StatusCode::NOT_FOUND, "MIDI file not found".to_string()));
    }

    // Refresh the file expiration
    {
        let mut expirations = state.file_expirations.lock().unwrap();
        expirations.insert(request.filename, Instant::now());
    }

    // Create MIDI processor
    let processor = ::desmos_midi::midi::MidiProcessor::with_soundfont_dir(
        state.soundfont_dir.to_str().unwrap(),
    );

    // Process the MIDI file with soundfonts
    let song = processor
        .process_with_soundfonts(file_path.to_str().unwrap(), request.soundfonts)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to process MIDI file: {}", e),
            )
        })?;

    // Convert to Desmos formula
    let formula = song.to_piecewise_function();

    Ok(Json(ConversionResponse { formula }))
}

// Handler for refreshing file expiration
async fn refresh_file_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RefreshFileRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Check if the file exists
    let file_path = state.temp_dir.join(&request.filename);
    if !file_path.exists() {
        return Err((StatusCode::NOT_FOUND, "MIDI file not found".to_string()));
    }

    // Refresh the file expiration
    {
        let mut expirations = state.file_expirations.lock().unwrap();
        expirations.insert(request.filename.clone(), Instant::now());
    }

    Ok(Json(serde_json::json!({
        "status": "ok",
        "message": "File expiration refreshed",
        "filename": request.filename,
        "expires_in_minutes": state.config.file_expiration_minutes
    })))
}

// New handler for getting files
async fn get_file_handler(
    State(state): State<Arc<AppState>>,
    Path(filename): Path<String>,
) -> Result<Response, (StatusCode, String)> {
    // Check if the file exists
    let file_path = state.temp_dir.join(&filename);
    if !file_path.exists() {
        return Err((StatusCode::NOT_FOUND, "File not found".to_string()));
    }

    // Refresh the file expiration
    {
        let mut expirations = state.file_expirations.lock().unwrap();
        expirations.insert(filename.clone(), Instant::now());
    }

    // Read the file
    let file_contents = fs::read(&file_path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read file: {}", e),
        )
    })?;

    // Determine content type based on file extension
    let content_type = match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("mid") | Some("midi") => "audio/midi",
        Some("wav") => "audio/wav",
        Some("mp3") => "audio/mpeg",
        Some("txt") => "text/plain",
        Some("json") => "application/json",
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        _ => "application/octet-stream",
    };

    // Build response with appropriate headers
    let mut response = Response::new(file_contents.into());
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_str(content_type).unwrap(),
    );

    // Add content disposition header for download
    let disposition = format!("attachment; filename=\"{}\"", filename);
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        header::HeaderValue::from_str(&disposition)
            .unwrap_or_else(|_| header::HeaderValue::from_static("attachment")),
    );

    Ok(response)
}

// Handler for listing available soundfonts
async fn list_soundfonts_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Read the soundfont directory
    let mut entries = fs::read_dir(&state.soundfont_dir).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read soundfont directory: {}", e),
        )
    })?;

    // Collect all .txt files
    let mut soundfonts = Vec::new();
    while let Some(entry) = entries.next_entry().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read directory entry: {}", e),
        )
    })? {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "txt") {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                soundfonts.push(name.to_string());
            }
        }
    }

    Ok(Json(serde_json::json!({ "soundfonts": soundfonts })))
}

// Handler for saving soundfonts
async fn save_soundfont_handler(
    Path(filename): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(soundfont): Json<Vec<f32>>,
) -> Result<Response, (StatusCode, String)> {
    // Ensure filename has .txt extension
    let filename = if !filename.ends_with(".txt") {
        format!("{}.txt", filename)
    } else {
        filename
    };

    // Create the file path
    let file_path = state.soundfont_dir.join(&filename);

    // Convert soundfont weights to string
    let content = soundfont
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(",");

    // Write to file
    fs::write(&file_path, content).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to write soundfont file: {}", e),
        )
    })?;

    let json = serde_json::json!({
        "status": "ok",
        "message": "Soundfont saved successfully",
        "filename": filename
    });

    Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(json.to_string()))
        .unwrap())
}

// Handler for analyzing WAV files
async fn harmonic_info_handler(
    State(state): State<Arc<AppState>>,
    Path(filename): Path<String>,
    Query(params): Query<HarmonicParams>,
) -> Result<Json<HarmonicResponse>, (StatusCode, String)> {
    let config = Arc::clone(&state.config);
    let limits = &config.limits;

    // Get parameters with defaults and limits
    let samples = params
        .samples
        .unwrap_or(8192)
        .clamp(limits.min_samples, limits.max_samples);
    let start_time = params
        .start_time
        .unwrap_or(0.0)
        .clamp(limits.min_start_time, limits.max_start_time);
    let base_freq = params
        .base_freq
        .unwrap_or(440.0)
        .clamp(limits.min_base_freq, limits.max_base_freq);
    let harmonics = params
        .harmonics
        .unwrap_or(16)
        .clamp(limits.min_harmonics, limits.max_harmonics);
    let boost = params
        .boost
        .unwrap_or(1.0)
        .clamp(limits.min_boost, limits.max_boost);

    let analysis_config = AnalysisConfig {
        samples,
        start_time,
        base_freq,
        num_harmonics: harmonics,
        boost,
    };

    // Check if the file exists
    let file_path = state.temp_dir.join(&filename);
    if !file_path.exists() {
        return Err((StatusCode::NOT_FOUND, "WAV file not found".to_string()));
    }

    // Read and analyze the WAV file
    let wav_data = read_wav_file(&file_path).map_err(|e| match e {
        AudioError::Io(io_err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read WAV file: {}", io_err),
        ),
        AudioError::WavParse(msg) => (
            StatusCode::BAD_REQUEST,
            format!("Invalid WAV file: {}", msg),
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error reading WAV file: {}", e),
        ),
    })?;

    let harmonics = analyze_harmonics(&wav_data, &analysis_config).map_err(|e| match e {
        AudioError::InvalidParams(msg) => (StatusCode::BAD_REQUEST, msg),
        AudioError::ProcessingError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error analyzing WAV file: {}", e),
        ),
    })?;

    Ok(Json(HarmonicResponse { harmonics }))
}

// Load configuration from config.toml
fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let mut file = File::open("config.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(toml::from_str(&contents)?)
}
