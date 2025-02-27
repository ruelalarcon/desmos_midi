use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
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
use uuid::Uuid;

// Constants for file expiration
const FILE_EXPIRATION_MINUTES: u64 = 10;
const FILE_REFRESH_THRESHOLD_MINUTES: u64 = 5;
const DEFAULT_PORT: u16 = 8573;

// App state
#[derive(Clone)]
struct AppState {
    temp_dir: PathBuf,
    soundfont_dir: PathBuf,
    file_expirations: Arc<Mutex<HashMap<String, Instant>>>,
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
    midi_file_id: String,
    soundfonts: Vec<String>,
}

// Request for refreshing file expiration
#[derive(Deserialize)]
struct RefreshFileRequest {
    file_id: String,
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

    // Parse port from command line arguments
    let port = parse_port_from_args().unwrap_or(DEFAULT_PORT);

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
    });

    // Create static directory if it doesn't exist
    let static_dir = PathBuf::from("src/web/static");
    if !static_dir.exists() {
        fs::create_dir_all(&static_dir).await.unwrap();
    }

    // Start background task for file cleanup
    let cleanup_state = state.clone();
    task::spawn(async move {
        run_file_cleanup(cleanup_state).await;
    });

    // Create router
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/upload", post(upload_handler))
        .route("/midi-info/{file_id}", get(midi_info_handler))
        .route("/convert", post(convert_handler))
        .route("/soundfonts", get(list_soundfonts_handler))
        .route("/refresh-file", post(refresh_file_handler))
        .nest_service("/static", ServeDir::new(StdPath::new("src/web/static")))
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("Listening on http://localhost:{}", port);

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Parse port from command line arguments
// Format: --port XXXX or -p XXXX
fn parse_port_from_args() -> Option<u16> {
    let args: Vec<String> = env::args().collect();

    for i in 0..args.len() - 1 {
        if args[i] == "--port" || args[i] == "-p" {
            if let Ok(port) = args[i + 1].parse::<u16>() {
                return Some(port);
            }
        }
    }

    None
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
            let expiration_duration = Duration::from_secs(FILE_EXPIRATION_MINUTES * 60);

            expirations.retain(|file_id, expiration_time| {
                let is_expired = now.duration_since(*expiration_time) >= expiration_duration;
                if is_expired {
                    expired_files.push(file_id.clone());
                    false
                } else {
                    true
                }
            });
        }

        // Remove expired files
        for file_id in expired_files {
            let file_path = state.temp_dir.join(format!("{}.mid", file_id));
            if let Err(e) = fs::remove_file(&file_path).await {
                tracing::warn!(
                    "Failed to remove expired file {}: {}",
                    file_path.display(),
                    e
                );
            } else {
                tracing::info!("Removed expired file: {}", file_id);
            }
        }
    }
}

// Handler for the index page
async fn index_handler() -> impl IntoResponse {
    let html = include_str!("static/index.html");
    Html(html)
}

// Handler for uploading MIDI files
async fn upload_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Generate a unique ID for the file
    let file_id = Uuid::new_v4().to_string();
    let mut file_path = None;

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
                .unwrap_or_else(|| "unknown.mid".to_string());

            // Only accept MIDI files
            if !file_name.ends_with(".mid") && !file_name.ends_with(".midi") {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Only MIDI files (.mid, .midi) are accepted".to_string(),
                ));
            }

            // Create the file path
            let path = state.temp_dir.join(format!("{}.mid", file_id));
            file_path = Some(path.clone());

            // Get the file data
            let data = field.bytes().await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to read file: {}", e),
                )
            })?;

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
                expirations.insert(file_id.clone(), Instant::now());
            }
        }
    }

    // Return the file ID
    match file_path {
        Some(_) => Ok(Json(serde_json::json!({
            "file_id": file_id,
            "expires_in_minutes": FILE_EXPIRATION_MINUTES,
            "refresh_threshold_minutes": FILE_REFRESH_THRESHOLD_MINUTES
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
    Path(file_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Check if the file exists
    let file_path = state.temp_dir.join(format!("{}.mid", file_id));
    if !file_path.exists() {
        return Err((StatusCode::NOT_FOUND, "MIDI file not found".to_string()));
    }

    // Refresh the file expiration
    {
        let mut expirations = state.file_expirations.lock().unwrap();
        expirations.insert(file_id, Instant::now());
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
    let file_path = state.temp_dir.join(format!("{}.mid", request.midi_file_id));
    if !file_path.exists() {
        return Err((StatusCode::NOT_FOUND, "MIDI file not found".to_string()));
    }

    // Refresh the file expiration
    {
        let mut expirations = state.file_expirations.lock().unwrap();
        expirations.insert(request.midi_file_id, Instant::now());
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
    let file_path = state.temp_dir.join(format!("{}.mid", request.file_id));
    if !file_path.exists() {
        return Err((StatusCode::NOT_FOUND, "MIDI file not found".to_string()));
    }

    // Refresh the file expiration
    {
        let mut expirations = state.file_expirations.lock().unwrap();
        expirations.insert(request.file_id.clone(), Instant::now());
    }

    Ok(Json(serde_json::json!({
        "status": "ok",
        "message": "File expiration refreshed",
        "file_id": request.file_id,
        "expires_in_minutes": FILE_EXPIRATION_MINUTES
    })))
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
