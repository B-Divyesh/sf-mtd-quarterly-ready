use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Path, State},
    http::{header, HeaderName, HeaderValue, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};
use std::{
    collections::{HashMap, VecDeque},
    env,
    net::SocketAddr,
    path::{Path as FsPath, PathBuf},
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{fs, signal, sync::Mutex};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::{error, info};
use uuid::Uuid;

const BUILD_SHA: &str = match option_env!("BUILD_SHA") {
    Some(value) => value,
    None => "dev",
};
const MAX_DOCUMENT_BYTES: usize = 5 * 1024 * 1024;

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    key: [u8; 32],
    limits: Arc<Mutex<HashMap<String, VecDeque<Instant>>>>,
}

#[derive(Serialize)]
struct Health<'a> {
    status: &'a str,
    build_sha: &'a str,
}

#[derive(Deserialize)]
struct WorkspaceDocument {
    document: Value,
}

#[derive(Serialize)]
struct ShareResult {
    token: String,
    expires_at: u64,
}

#[derive(Debug)]
struct ApiError(StatusCode, &'static str);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let data_dir = PathBuf::from(env::var("DATA_DIR").unwrap_or_else(|_| "./data".into()));
    let frontend_dir = PathBuf::from(env::var("FRONTEND_DIR").unwrap_or_else(|_| "./dist".into()));
    fs::create_dir_all(&data_dir)
        .await
        .expect("create data directory");
    let (key, generated) = load_or_create_key(&data_dir.join("quarterly-ready.key"))
        .await
        .expect("load encryption key");
    let database_url = format!(
        "sqlite://{}?mode=rwc",
        data_dir.join("quarterly-ready.sqlite3").display()
    );
    let db = SqlitePoolOptions::new()
        .max_connections(8)
        .connect(&database_url)
        .await
        .expect("open database");
    migrate(&db).await.expect("run database migrations");
    cleanup_expired_shares(&db)
        .await
        .expect("clean expired accountant links");
    let cleanup_db = db.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60 * 60));
        loop {
            interval.tick().await;
            if let Err(error) = cleanup_expired_shares(&cleanup_db).await {
                error!(%error, "expired_share_cleanup_failed");
            }
        }
    });
    let state = AppState {
        db,
        key,
        limits: Arc::new(Mutex::new(HashMap::new())),
    };
    let app = build_router(state, frontend_dir);

    info!(
        port,
        build_sha = BUILD_SHA,
        encryption_key = if generated { "generated" } else { "persisted" },
        "quarterly_ready_started"
    );
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .expect("bind server");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .expect("serve application");
}

fn build_router(state: AppState, frontend_dir: PathBuf) -> Router {
    let index = frontend_dir.join("index.html");
    Router::new()
        .route("/health", get(health))
        .route("/api/workspace", get(get_workspace).put(put_workspace))
        .route("/api/share", post(create_share))
        .route("/api/share/:token", get(get_share))
        .route("/api/page-view", post(page_view))
        .fallback_service(ServeDir::new(frontend_dir).fallback(ServeFile::new(index)))
        .layer(DefaultBodyLimit::max(MAX_DOCUMENT_BYTES))
        .layer(middleware::from_fn(security_headers))
        .layer(middleware::from_fn_with_state(state.clone(), rate_limit))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health() -> Json<Health<'static>> {
    Json(Health {
        status: "ok",
        build_sha: BUILD_SHA,
    })
}

async fn get_workspace(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<Json<Value>, ApiError> {
    let id = workspace_id(&request)?;
    let row = sqlx::query("SELECT payload FROM workspaces WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal)?;
    let Some(row) = row else {
        return Err(ApiError(
            StatusCode::NOT_FOUND,
            "No records have been saved for this workspace.",
        ));
    };
    let payload: Vec<u8> = row.get("payload");
    let document = decrypt_json(&state.key, &payload)?;
    Ok(Json(json!({ "document": document })))
}

async fn put_workspace(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<Json<Value>, ApiError> {
    let id = workspace_id(&request)?;
    let bytes = axum::body::to_bytes(request.into_body(), MAX_DOCUMENT_BYTES)
        .await
        .map_err(|_| {
            ApiError(
                StatusCode::BAD_REQUEST,
                "The records could not be read. Try saving again.",
            )
        })?;
    let input: WorkspaceDocument = serde_json::from_slice(&bytes).map_err(|_| {
        ApiError(
            StatusCode::BAD_REQUEST,
            "The records are not valid JSON. Check the file and try again.",
        )
    })?;
    validate_document(&input.document)?;
    let encrypted = encrypt_json(&state.key, &input.document)?;
    let now = unix_now();
    sqlx::query("INSERT INTO workspaces(id, payload, updated_at) VALUES(?, ?, ?) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload, updated_at=excluded.updated_at")
        .bind(&id).bind(encrypted).bind(now as i64).execute(&state.db).await.map_err(internal)?;
    write_audit(&state, &id, "records_saved", &bytes).await?;
    Ok(Json(json!({ "saved": true, "updated_at": now })))
}

async fn create_share(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<(StatusCode, Json<ShareResult>), ApiError> {
    let id = workspace_id(&request)?;
    let bytes = axum::body::to_bytes(request.into_body(), MAX_DOCUMENT_BYTES)
        .await
        .map_err(|_| {
            ApiError(
                StatusCode::BAD_REQUEST,
                "The accountant pack could not be read. Try again.",
            )
        })?;
    let input: WorkspaceDocument = serde_json::from_slice(&bytes).map_err(|_| {
        ApiError(
            StatusCode::BAD_REQUEST,
            "The accountant pack is not valid. Review it and try again.",
        )
    })?;
    validate_document(&input.document)?;
    let token = Uuid::new_v4().simple().to_string();
    let expires_at = unix_now() + 30 * 24 * 60 * 60;
    let encrypted = encrypt_json(&state.key, &input.document)?;
    sqlx::query("INSERT INTO shares(token, workspace_id, payload, expires_at) VALUES(?, ?, ?, ?)")
        .bind(&token)
        .bind(&id)
        .bind(encrypted)
        .bind(expires_at as i64)
        .execute(&state.db)
        .await
        .map_err(internal)?;
    write_audit(&state, &id, "accountant_link_created", token.as_bytes()).await?;
    Ok((StatusCode::CREATED, Json(ShareResult { token, expires_at })))
}

async fn get_share(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Json<Value>, ApiError> {
    if token.len() != 32 || !token.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "This accountant link is not valid.",
        ));
    }
    let row = sqlx::query("SELECT payload, expires_at FROM shares WHERE token = ?")
        .bind(&token)
        .fetch_optional(&state.db)
        .await
        .map_err(internal)?;
    let Some(row) = row else {
        return Err(ApiError(
            StatusCode::NOT_FOUND,
            "This accountant link was not found.",
        ));
    };
    let expires_at: i64 = row.get("expires_at");
    if expires_at < unix_now() as i64 {
        return Err(ApiError(
            StatusCode::GONE,
            "This accountant link has expired. Ask for a new link.",
        ));
    }
    let payload: Vec<u8> = row.get("payload");
    Ok(Json(
        json!({ "document": decrypt_json(&state.key, &payload)?, "expires_at": expires_at }),
    ))
}

async fn page_view(State(state): State<AppState>) -> Result<StatusCode, ApiError> {
    let day = unix_now() / 86_400;
    sqlx::query("INSERT INTO page_views(day, count) VALUES(?, 1) ON CONFLICT(day) DO UPDATE SET count=count+1")
        .bind(day as i64).execute(&state.db).await.map_err(internal)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn rate_limit(State(state): State<AppState>, request: Request<Body>, next: Next) -> Response {
    if request.uri().path() == "/health" {
        return next.run(request).await;
    }
    let write_request = request.method() != axum::http::Method::GET;
    let key = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or("direct")
        .to_owned()
        + if write_request { ":write" } else { ":read" };
    let allowance = if write_request { 12 } else { 40 };
    let now = Instant::now();
    let mut limits = state.limits.lock().await;
    let hits = limits.entry(key).or_default();
    while hits
        .front()
        .is_some_and(|t| now.duration_since(*t) >= Duration::from_secs(1))
    {
        hits.pop_front();
    }
    if hits.len() >= allowance {
        drop(limits);
        let mut response = (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({ "error": "Too many requests. Wait one second and try again." })),
        )
            .into_response();
        response
            .headers_mut()
            .insert(header::RETRY_AFTER, HeaderValue::from_static("1"));
        return response;
    }
    hits.push_back(now);
    drop(limits);
    next.run(request).await
}

async fn security_headers(request: Request<Body>, next: Next) -> Response {
    let immutable_asset = request.uri().path().starts_with("/assets/");
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    headers.insert(HeaderName::from_static("content-security-policy"), HeaderValue::from_static("default-src 'self'; img-src 'self' data: blob:; style-src 'self'; script-src 'self'; connect-src 'self' https://api.sociobot.in; object-src 'none'; base-uri 'self'; form-action 'self' https://api.sociobot.in; frame-ancestors 'none'"));
    if immutable_asset {
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=31536000, immutable"),
        );
    }
    response
}

fn workspace_id(request: &Request<Body>) -> Result<String, ApiError> {
    let id = request
        .headers()
        .get("x-workspace-id")
        .and_then(|v| v.to_str().ok())
        .ok_or(ApiError(
            StatusCode::BAD_REQUEST,
            "This browser workspace is missing. Reload the page and try again.",
        ))?;
    Uuid::parse_str(id).map_err(|_| {
        ApiError(
            StatusCode::BAD_REQUEST,
            "This browser workspace is not valid.",
        )
    })?;
    Ok(id.to_owned())
}

fn validate_document(value: &Value) -> Result<(), ApiError> {
    let object = value.as_object().ok_or(ApiError(
        StatusCode::UNPROCESSABLE_ENTITY,
        "The records must be a document.",
    ))?;
    let transactions = object
        .get("transactions")
        .and_then(Value::as_array)
        .ok_or(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "The records need a transactions list.",
        ))?;
    if transactions.len() > 10_000 {
        return Err(ApiError(
            StatusCode::PAYLOAD_TOO_LARGE,
            "A workspace can hold up to 10,000 transactions.",
        ));
    }
    Ok(())
}

fn encrypt_json(key: &[u8; 32], value: &Value) -> Result<Vec<u8>, ApiError> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| {
        ApiError(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Encryption could not start.",
        )
    })?;
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    let plain = serde_json::to_vec(value).map_err(internal)?;
    let encrypted = cipher
        .encrypt(Nonce::from_slice(&nonce), plain.as_ref())
        .map_err(|_| {
            ApiError(
                StatusCode::INTERNAL_SERVER_ERROR,
                "The records could not be encrypted.",
            )
        })?;
    let mut stored = nonce.to_vec();
    stored.extend(encrypted);
    Ok(B64.encode(stored).into_bytes())
}

fn decrypt_json(key: &[u8; 32], stored: &[u8]) -> Result<Value, ApiError> {
    let decoded = B64.decode(stored).map_err(|_| {
        ApiError(
            StatusCode::INTERNAL_SERVER_ERROR,
            "The saved records could not be decoded.",
        )
    })?;
    if decoded.len() < 13 {
        return Err(ApiError(
            StatusCode::INTERNAL_SERVER_ERROR,
            "The saved records are incomplete.",
        ));
    }
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| {
        ApiError(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Decryption could not start.",
        )
    })?;
    let plain = cipher
        .decrypt(Nonce::from_slice(&decoded[..12]), &decoded[12..])
        .map_err(|_| {
            ApiError(
                StatusCode::INTERNAL_SERVER_ERROR,
                "The saved records could not be decrypted.",
            )
        })?;
    serde_json::from_slice(&plain).map_err(internal)
}

async fn write_audit(
    state: &AppState,
    workspace: &str,
    action: &str,
    detail: &[u8],
) -> Result<(), ApiError> {
    let previous =
        sqlx::query("SELECT hash FROM audit_log WHERE workspace_id = ? ORDER BY id DESC LIMIT 1")
            .bind(workspace)
            .fetch_optional(&state.db)
            .await
            .map_err(internal)?
            .map(|row| row.get::<String, _>("hash"))
            .unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(previous.as_bytes());
    hasher.update(action.as_bytes());
    hasher.update(detail);
    let hash = format!("{:x}", hasher.finalize());
    sqlx::query("INSERT INTO audit_log(workspace_id, action, created_at, hash) VALUES(?, ?, ?, ?)")
        .bind(workspace)
        .bind(action)
        .bind(unix_now() as i64)
        .bind(hash)
        .execute(&state.db)
        .await
        .map_err(internal)?;
    Ok(())
}

async fn load_or_create_key(path: &FsPath) -> Result<([u8; 32], bool), std::io::Error> {
    if let Ok(bytes) = fs::read(path).await {
        if bytes.len() == 32 {
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            return Ok((key, false));
        }
    }
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    fs::write(path, key).await?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).await?;
    }
    Ok((key, true))
}

async fn migrate(db: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("CREATE TABLE IF NOT EXISTS workspaces(id TEXT PRIMARY KEY, payload BLOB NOT NULL, updated_at INTEGER NOT NULL)").execute(db).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS shares(token TEXT PRIMARY KEY, workspace_id TEXT NOT NULL, payload BLOB NOT NULL, expires_at INTEGER NOT NULL)").execute(db).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS audit_log(id INTEGER PRIMARY KEY AUTOINCREMENT, workspace_id TEXT NOT NULL, action TEXT NOT NULL, created_at INTEGER NOT NULL, hash TEXT NOT NULL)").execute(db).await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS page_views(day INTEGER PRIMARY KEY, count INTEGER NOT NULL)",
    )
    .execute(db)
    .await?;
    Ok(())
}

async fn cleanup_expired_shares(db: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM shares WHERE expires_at < ?")
        .bind(unix_now() as i64)
        .execute(db)
        .await?;
    Ok(())
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn internal<E: std::fmt::Display>(error: E) -> ApiError {
    error!(%error, "request_failed");
    ApiError(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Quarterly Ready could not finish that request. Try again.",
    )
}

async fn shutdown_signal() {
    let ctrl_c = async { signal::ctrl_c().await.expect("install Ctrl+C handler") };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("install signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
    info!("quarterly_ready_stopping");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claim_encrypted_storage() {
        let key = [7u8; 32];
        let document = json!({"transactions":[{"description":"Maths lesson"}]});
        let encrypted = encrypt_json(&key, &document).unwrap();
        assert!(!String::from_utf8_lossy(&encrypted).contains("Maths lesson"));
        assert_eq!(decrypt_json(&key, &encrypted).unwrap(), document);
    }

    #[test]
    fn validates_transaction_document() {
        assert!(validate_document(&json!({"transactions":[]})).is_ok());
        assert!(validate_document(&json!({"items":[]})).is_err());
    }

    #[tokio::test]
    async fn claim_anonymous_page_count() {
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let columns = sqlx::query("PRAGMA table_info(page_views)")
            .fetch_all(&db)
            .await
            .unwrap();
        let names: Vec<String> = columns.into_iter().map(|row| row.get("name")).collect();
        assert_eq!(names, vec!["day", "count"]);
    }

    #[tokio::test]
    async fn claim_hash_chained_audit_log() {
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let state = AppState {
            db,
            key: [9u8; 32],
            limits: Arc::new(Mutex::new(HashMap::new())),
        };
        write_audit(&state, "workspace", "first", b"one")
            .await
            .unwrap();
        write_audit(&state, "workspace", "second", b"two")
            .await
            .unwrap();
        let hashes = sqlx::query("SELECT hash FROM audit_log ORDER BY id")
            .fetch_all(&state.db)
            .await
            .unwrap();
        let first: String = hashes[0].get("hash");
        let second: String = hashes[1].get("hash");
        let mut expected = Sha256::new();
        expected.update(first.as_bytes());
        expected.update(b"second");
        expected.update(b"two");
        assert_eq!(second, format!("{:x}", expected.finalize()));
    }
}
