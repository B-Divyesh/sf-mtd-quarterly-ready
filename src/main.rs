use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Path, Query, State},
    handler::Handler,
    http::{header, HeaderMap, HeaderName, HeaderValue, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::{get, get_service, post},
    Json, Router,
};
use base64::{
    engine::general_purpose::{STANDARD as B64, URL_SAFE_NO_PAD},
    Engine,
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use rand::{rngs::OsRng, RngCore};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{sqlite::SqlitePoolOptions, Row, Sqlite, SqlitePool, Transaction};
use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    path::{Path as FsPath, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    signal,
    sync::Mutex,
};
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::KeyExtractor, GovernorError, GovernorLayer,
};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::{error, info, warn};
use uuid::Uuid;

const BUILD_SHA: &str = match option_env!("BUILD_SHA") {
    Some(value) => value,
    None => "dev",
};
const MAX_DOCUMENT_BYTES: usize = 5 * 1024 * 1024;
const SHARE_EXPIRY_SECONDS: u64 = 30 * 24 * 60 * 60;
const SESSION_EXPIRY_SECONDS: u64 = 14 * 24 * 60 * 60;
const AUTH_STATE_EXPIRY_SECONDS: u64 = 10 * 60;
const SESSION_COOKIE: &str = "quarterly_ready_session";
const AUTH_STATE_COOKIE: &str = "quarterly_ready_auth_state";
const SAFE_FIXTURE_TOKEN: &str = "quarterly-ready-safe-no-charge-fixture-v1";
const SAFE_FIXTURE_BUSINESS: &str = "Quarterly Ready safe QA fixture";
const SQLITE_BUSY_TIMEOUT_MS: u64 = 30_000;
const RATE_LIMIT_REFILL_SECONDS: u64 = 60;
const READ_RATE_LIMIT_BURST: u32 = 40;
const WRITE_RATE_LIMIT_BURST: u32 = 12;
const PRODUCT_SLUGS: [&str; 2] = ["mtd-quarterly-ready", "mtd-quarterly-ready-annual"];
const CATEGORIES: [&str; 7] = [
    "Sales",
    "Rent and rates",
    "Travel",
    "Office costs",
    "Professional fees",
    "Repairs",
    "Other",
];

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    key: [u8; 32],
    database_path: PathBuf,
    snapshot_path: PathBuf,
    write_lock: Arc<Mutex<()>>,
    client: reqwest::Client,
    billing_base_url: String,
    auth: Option<AuthConfig>,
    hmrc_integration: Option<ApprovedIntegration>,
    safe_qa_fixtures: bool,
}

/// Sociobot Entra CIAM is a public OIDC client. The client ID and issuer are
/// runtime configuration, rather than source-controlled credentials. PKCE
/// means this product does not require an OAuth client secret.
#[derive(Clone)]
struct AuthConfig {
    issuer: String,
    client_id: String,
    redirect_uri: String,
}

#[derive(Deserialize)]
struct OidcDiscovery {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    jwks_uri: String,
}

#[derive(Deserialize)]
struct OidcJwks {
    keys: Vec<OidcJwk>,
}

#[derive(Deserialize)]
struct OidcJwk {
    kid: Option<String>,
    kty: String,
    n: Option<String>,
    e: Option<String>,
}

#[derive(Clone, Deserialize)]
struct IdTokenClaims {
    sub: String,
    iss: String,
    exp: usize,
    nonce: Option<String>,
    email: Option<String>,
    name: Option<String>,
    preferred_username: Option<String>,
}

#[derive(Clone)]
struct AuthenticatedUser {
    subject: String,
    display_name: String,
    email: Option<String>,
}

#[derive(Clone)]
struct ApprovedIntegration {
    url: String,
    token: String,
    mode: IntegrationMode,
    taxpayer_consent: Option<TaxpayerConsent>,
}

#[derive(Clone)]
struct TaxpayerConsent {
    authorize_url: String,
    token_url: String,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    provider_name: String,
    provider_approval_reference: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IntegrationMode {
    ApprovedProvider,
    HmrcSandboxNoFiling,
}

impl IntegrationMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::ApprovedProvider => "approved_provider",
            Self::HmrcSandboxNoFiling => "hmrc_sandbox_no_filing",
        }
    }
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    build_sha: &'static str,
    safe_qa_fixtures: bool,
    hmrc_integration_configured: bool,
    hmrc_integration_mode: &'static str,
    hmrc_taxpayer_consent_required: bool,
    hmrc_provider_name: Option<String>,
    accounts_configured: bool,
}

#[derive(Serialize)]
struct AuthSession {
    configured: bool,
    authenticated: bool,
    user: Option<AccountUser>,
    businesses: Vec<Business>,
}

#[derive(Serialize)]
struct AccountUser {
    display_name: String,
    email: Option<String>,
}

#[derive(Serialize, Clone)]
struct Business {
    id: String,
    name: String,
    role: String,
}

#[derive(Deserialize)]
struct CreateBusiness {
    name: String,
}

#[derive(Deserialize)]
struct AuthCallback {
    state: String,
    code: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct OidcTokenResponse {
    id_token: String,
}

#[derive(Serialize)]
struct AuthStart {
    authorization_url: String,
}

#[derive(Deserialize)]
struct AccountQuarterWrite {
    document: Value,
    migration_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct AccountQuarterResult {
    document: Option<Value>,
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

#[derive(Deserialize)]
struct SubmissionRequest {
    document: Value,
    review_confirmed: bool,
}

#[derive(Deserialize)]
struct ConsentCallback {
    state: String,
    code: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
struct ConsentStart {
    authorization_url: String,
}

#[derive(Serialize)]
struct ConsentStatus {
    consented: bool,
    expires_at: Option<u64>,
}

#[derive(Deserialize)]
struct ProviderTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
}

#[derive(Deserialize)]
struct LicenceVerdict {
    valid: bool,
}

#[derive(Serialize)]
struct SubmissionResult {
    submission_id: String,
    status: &'static str,
    files_with_hmrc: bool,
}

#[derive(Debug)]
struct ApiError(StatusCode, &'static str);

#[derive(Clone)]
struct ForwardedClientKeyExtractor;

impl KeyExtractor for ForwardedClientKeyExtractor {
    type Key = String;

    fn extract<T>(&self, request: &Request<T>) -> Result<Self::Key, GovernorError> {
        Ok(request
            .headers()
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(',').next())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .or_else(|| {
                request
                    .headers()
                    .get("x-quarterly-ready-client")
                    .and_then(|value| value.to_str().ok())
                    .map(str::trim)
                    .filter(|value| Uuid::parse_str(value).is_ok())
                    .map(|value| format!("browser:{value}"))
            })
            .unwrap_or_else(|| "direct".to_owned()))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(default_log_filter())
        .init();

    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let data_dir = PathBuf::from(env::var("DATA_DIR").unwrap_or_else(|_| "./data".into()));
    // SQLite uses byte-range locks that Azure Files does not reliably support.
    // Keep its live file on local disk and synchronously persist a complete
    // snapshot to the mounted share after every acknowledged mutation.
    let database_dir =
        PathBuf::from(env::var("DATABASE_DIR").unwrap_or_else(|_| "/tmp/quarterly-ready".into()));
    let frontend_dir = PathBuf::from(env::var("FRONTEND_DIR").unwrap_or_else(|_| "./dist".into()));
    fs::create_dir_all(&data_dir)
        .await
        .expect("create data directory");
    fs::create_dir_all(&database_dir)
        .await
        .expect("create database directory");
    let snapshot_path = data_dir.join("quarterly-ready.snapshot.sqlite3");
    let database_path = database_dir.join("quarterly-ready.sqlite3");
    restore_database_snapshot(&snapshot_path, &database_path)
        .await
        .expect("restore legacy database snapshot");
    let (key, generated) = load_or_create_key(&data_dir.join("quarterly-ready.key"))
        .await
        .expect("load encryption key");
    let database_url = format!("sqlite://{}?mode=rwc", database_path.display());
    let db = SqlitePoolOptions::new()
        // The Container App runs one active replica. Writes are serialized,
        // committed locally, copied to /data, synced, then acknowledged.
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("open database");
    sqlx::query(&format!("PRAGMA busy_timeout = {SQLITE_BUSY_TIMEOUT_MS}"))
        .execute(&db)
        .await
        .expect("configure database busy timeout");
    sqlx::query("PRAGMA journal_mode = DELETE")
        .execute(&db)
        .await
        .expect("configure database journal");
    sqlx::query("PRAGMA synchronous = FULL")
        .execute(&db)
        .await
        .expect("configure database durability");
    migrate_with_retry(&db)
        .await
        .expect("run database migrations");
    cleanup_expired_shares(&db)
        .await
        .expect("clean expired accountant links");
    persist_database_snapshot(&database_path, &snapshot_path)
        .await
        .expect("persist startup database snapshot");
    let hmrc_integration = approved_integration_from_env();
    let auth = auth_config_from_env();
    let safe_qa_fixtures = safe_fixtures_enabled();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("build HTTP client");
    let state = AppState {
        db,
        key,
        database_path,
        snapshot_path,
        write_lock: Arc::new(Mutex::new(())),
        client,
        billing_base_url: env::var("SOCIOBOT_BILLING_URL")
            .unwrap_or_else(|_| "https://api.sociobot.in/api/v1".into()),
        auth,
        hmrc_integration,
        safe_qa_fixtures,
    };
    let cleanup_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60 * 60));
        loop {
            interval.tick().await;
            let _write = cleanup_state.write_lock.lock().await;
            if let Err(error) = cleanup_expired_shares(&cleanup_state.db).await {
                error!(%error, "expired_share_cleanup_failed");
            } else if let Err(error) = persist_database_snapshot(
                &cleanup_state.database_path,
                &cleanup_state.snapshot_path,
            )
            .await
            {
                error!(%error, "expired_share_cleanup_persist_failed");
            }
        }
    });
    let integration_configured = state.hmrc_integration.is_some();
    let integration_mode = state
        .hmrc_integration
        .as_ref()
        .map(|integration| integration.mode.as_str())
        .unwrap_or("not_configured");
    let accounts_configured = state.auth.is_some();
    let app = build_router(state, frontend_dir);

    info!(
        port,
        build_sha = BUILD_SHA,
        encryption_key = if generated { "generated" } else { "persisted" },
        hmrc_integration = if integration_configured {
            integration_mode
        } else {
            "not_configured"
        },
        safe_qa_fixtures,
        accounts = if accounts_configured {
            "configured"
        } else {
            "not_configured"
        },
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
    let read_limit = rate_limit_layer(READ_RATE_LIMIT_BURST);
    let write_limit = rate_limit_layer(WRITE_RATE_LIMIT_BURST);
    Router::new()
        .route("/health", get(health))
        .route(
            "/api/auth/session",
            get(auth_session.layer(read_limit.clone())),
        )
        .route(
            "/api/auth/start",
            post(auth_start.layer(write_limit.clone())),
        )
        .route(
            "/api/auth/callback",
            get(auth_callback.layer(write_limit.clone())),
        )
        .route(
            "/api/account",
            get(account_details.layer(read_limit.clone()))
                .delete(delete_account.layer(write_limit.clone())),
        )
        .route(
            "/api/account/export",
            get(export_account.layer(read_limit.clone())),
        )
        .route(
            "/api/businesses",
            post(create_business.layer(write_limit.clone())),
        )
        .route(
            "/api/businesses/:business_id/quarters/:quarter_start",
            get(get_account_quarter.layer(read_limit.clone()))
                .put(put_account_quarter.layer(write_limit.clone())),
        )
        .route(
            "/api/businesses/:business_id/quarters/:quarter_start/share",
            post(create_account_share.layer(write_limit.clone())),
        )
        .route(
            "/api/workspace",
            get(get_workspace.layer(read_limit.clone()))
                .put(put_workspace.layer(write_limit.clone())),
        )
        .route("/api/share", post(create_share.layer(write_limit.clone())))
        .route(
            "/api/share/:token",
            get(get_share.layer(read_limit.clone())),
        )
        .route(
            "/api/hmrc/submit",
            post(submit_to_hmrc.layer(write_limit.clone())),
        )
        .route(
            "/api/hmrc/consent",
            get(hmrc_consent_status.layer(read_limit.clone()))
                .post(start_hmrc_consent.layer(write_limit.clone())),
        )
        .route(
            "/api/hmrc/consent/callback",
            get(hmrc_consent_callback.layer(write_limit.clone())),
        )
        .route(
            "/api/qa/entitlement",
            get(safe_qa_entitlement.layer(read_limit)),
        )
        .route("/api/page-view", post(page_view.layer(write_limit)))
        .route_service("/", get_service(ServeFile::new(index.clone())))
        .route_service("/demo", get_service(ServeFile::new(index.clone())))
        .route_service("/records", get_service(ServeFile::new(index.clone())))
        .route_service("/account", get_service(ServeFile::new(index.clone())))
        .route_service("/privacy", get_service(ServeFile::new(index.clone())))
        .route_service("/terms", get_service(ServeFile::new(index.clone())))
        .route_service("/share/:token", get_service(ServeFile::new(index)))
        .fallback_service(
            ServeDir::new(frontend_dir.clone())
                .not_found_service(ServeFile::new(frontend_dir.join("404.html"))),
        )
        .layer(DefaultBodyLimit::max(MAX_DOCUMENT_BYTES))
        .layer(middleware::from_fn(security_headers))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

fn rate_limit_layer(
    burst_size: u32,
) -> GovernorLayer<ForwardedClientKeyExtractor, governor::middleware::StateInformationMiddleware> {
    let config = GovernorConfigBuilder::default()
        .per_second(RATE_LIMIT_REFILL_SECONDS)
        .burst_size(burst_size)
        .key_extractor(ForwardedClientKeyExtractor)
        .error_handler(rate_limit_error)
        .use_headers()
        .finish()
        .expect("rate limit configuration is valid");
    GovernorLayer {
        config: Arc::new(config),
    }
}

fn rate_limit_error(error: GovernorError) -> Response {
    match error {
        GovernorError::TooManyRequests { headers, .. } => {
            let mut response = (
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({
                    "error": "Too many requests. Wait for the Retry-After time and try again."
                })),
            )
                .into_response();
            if let Some(headers) = headers {
                response.headers_mut().extend(headers);
            }
            response
        }
        _ => ApiError(
            StatusCode::INTERNAL_SERVER_ERROR,
            "The request limit could not identify this connection. Try again.",
        )
        .into_response(),
    }
}

async fn health(State(state): State<AppState>) -> Json<Health> {
    let integration = state.hmrc_integration.as_ref();
    let integration_mode = integration
        .map(|configured| configured.mode.as_str())
        .unwrap_or("not_configured");
    Json(Health {
        status: "ok",
        build_sha: BUILD_SHA,
        safe_qa_fixtures: state.safe_qa_fixtures,
        hmrc_integration_configured: integration.is_some(),
        hmrc_integration_mode: integration_mode,
        hmrc_taxpayer_consent_required: integration
            .is_some_and(|configured| configured.taxpayer_consent.is_some()),
        hmrc_provider_name: integration
            .and_then(|configured| configured.taxpayer_consent.as_ref())
            .map(|consent| consent.provider_name.clone()),
        accounts_configured: state.auth.is_some(),
    })
}

fn auth_config_from_env() -> Option<AuthConfig> {
    auth_config_from_values(
        env::var("OIDC_ISSUER").ok(),
        env::var("OIDC_CLIENT_ID").ok(),
        env::var("OIDC_REDIRECT_URI").ok(),
    )
}

fn auth_config_from_values(
    issuer: Option<String>,
    client_id: Option<String>,
    redirect_uri: Option<String>,
) -> Option<AuthConfig> {
    let issuer = issuer?;
    let client_id = client_id?;
    let redirect_uri = redirect_uri
        .unwrap_or_else(|| "https://mtd-quarterly-ready.sociobot.in/api/auth/callback".into());
    if !issuer.starts_with("https://")
        || client_id.trim().is_empty()
        || !redirect_uri.starts_with("https://")
        || redirect_uri.contains('#')
    {
        return None;
    }
    Some(AuthConfig {
        issuer: issuer.trim_end_matches('/').to_owned(),
        client_id,
        redirect_uri,
    })
}

async fn oidc_discovery(
    client: &reqwest::Client,
    auth: &AuthConfig,
) -> Result<OidcDiscovery, ApiError> {
    let endpoint = format!("{}/.well-known/openid-configuration", auth.issuer);
    let response = client.get(endpoint).send().await.map_err(|_| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "The sign-in service could not be reached. Try again in a moment.",
        )
    })?;
    if !response.status().is_success() {
        return Err(ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "The sign-in service is not ready. Try again in a moment.",
        ));
    }
    let metadata: OidcDiscovery = response.json().await.map_err(|_| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "The sign-in service returned an unreadable setup. Try again in a moment.",
        )
    })?;
    if metadata.issuer.trim_end_matches('/') != auth.issuer
        || !metadata.authorization_endpoint.starts_with("https://")
        || !metadata.token_endpoint.starts_with("https://")
        || !metadata.jwks_uri.starts_with("https://")
    {
        return Err(ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "The sign-in service setup is not valid. Try again later.",
        ));
    }
    Ok(metadata)
}

fn random_url_token(byte_count: usize) -> String {
    let mut bytes = vec![0; byte_count];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn token_hash(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn pkce_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(hasher.finalize())
}

fn request_cookie(request: &Request<Body>, name: &str) -> Option<String> {
    request
        .headers()
        .get(header::COOKIE)
        .and_then(|header| header.to_str().ok())?
        .split(';')
        .map(str::trim)
        .find_map(|value| value.strip_prefix(&format!("{name}=")))
        .filter(|value| !value.is_empty() && value.len() <= 512)
        .map(str::to_owned)
}

/// The OIDC state is a high-entropy, one-time value. Comparing the complete
/// fixed-length value without an early exit also avoids turning this check into
/// an oracle for the browser-bound state cookie.
fn tokens_equal(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.as_bytes()
        .iter()
        .zip(right.as_bytes())
        .fold(0u8, |difference, (a, b)| difference | (a ^ b))
        == 0
}

async fn authenticated_user(
    state: &AppState,
    session_token: Option<String>,
) -> Result<AuthenticatedUser, ApiError> {
    let token = session_token.ok_or(ApiError(
        StatusCode::UNAUTHORIZED,
        "Sign in to access account records.",
    ))?;
    let row = sqlx::query("SELECT users.subject, users.display_name, users.email, sessions.expires_at FROM sessions JOIN users ON users.subject = sessions.subject WHERE sessions.token_hash = ?")
        .bind(token_hash(&token))
        .fetch_optional(&state.db)
        .await
        .map_err(internal)?
        .ok_or(ApiError(StatusCode::UNAUTHORIZED, "Your sign-in has expired. Sign in again."))?;
    let expires_at: i64 = row.get("expires_at");
    if expires_at <= unix_now() as i64 {
        return Err(ApiError(
            StatusCode::UNAUTHORIZED,
            "Your sign-in has expired. Sign in again.",
        ));
    }
    Ok(AuthenticatedUser {
        subject: row.get("subject"),
        display_name: row.get("display_name"),
        email: row.get("email"),
    })
}

async fn businesses_for_user(state: &AppState, subject: &str) -> Result<Vec<Business>, ApiError> {
    let rows = sqlx::query("SELECT businesses.id, businesses.name, memberships.role FROM memberships JOIN businesses ON businesses.id = memberships.business_id WHERE memberships.subject = ? ORDER BY businesses.created_at ASC")
        .bind(subject)
        .fetch_all(&state.db)
        .await
        .map_err(internal)?;
    Ok(rows
        .into_iter()
        .map(|row| Business {
            id: row.get("id"),
            name: row.get("name"),
            role: row.get("role"),
        })
        .collect())
}

async fn require_business_membership(
    state: &AppState,
    subject: &str,
    business_id: &str,
) -> Result<String, ApiError> {
    if Uuid::parse_str(business_id).is_err() {
        return Err(ApiError(StatusCode::NOT_FOUND, "Business not found."));
    }
    let role = sqlx::query("SELECT role FROM memberships WHERE business_id = ? AND subject = ?")
        .bind(business_id)
        .bind(subject)
        .fetch_optional(&state.db)
        .await
        .map_err(internal)?
        .map(|row| row.get::<String, _>("role"))
        .ok_or(ApiError(StatusCode::NOT_FOUND, "Business not found."))?;
    Ok(role)
}

async fn auth_session(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<Json<AuthSession>, ApiError> {
    let configured = state.auth.is_some();
    let session_token = request_cookie(&request, SESSION_COOKIE);
    let Ok(user) = authenticated_user(&state, session_token).await else {
        return Ok(Json(AuthSession {
            configured,
            authenticated: false,
            user: None,
            businesses: Vec::new(),
        }));
    };
    let businesses = businesses_for_user(&state, &user.subject).await?;
    Ok(Json(AuthSession {
        configured,
        authenticated: true,
        user: Some(AccountUser {
            display_name: user.display_name,
            email: user.email,
        }),
        businesses,
    }))
}

async fn account_details(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<Json<AuthSession>, ApiError> {
    let user = authenticated_user(&state, request_cookie(&request, SESSION_COOKIE)).await?;
    let businesses = businesses_for_user(&state, &user.subject).await?;
    Ok(Json(AuthSession {
        configured: state.auth.is_some(),
        authenticated: true,
        user: Some(AccountUser {
            display_name: user.display_name,
            email: user.email,
        }),
        businesses,
    }))
}

async fn auth_start(
    State(state): State<AppState>,
) -> Result<(HeaderMap, Json<AuthStart>), ApiError> {
    let auth = state.auth.as_ref().ok_or(ApiError(
        StatusCode::SERVICE_UNAVAILABLE,
        "Sign-in is not configured for this service yet.",
    ))?;
    let discovery = oidc_discovery(&state.client, auth).await?;
    let state_token = random_url_token(32);
    let nonce = random_url_token(32);
    let verifier = random_url_token(48);
    let expires_at = unix_now() + AUTH_STATE_EXPIRY_SECONDS;
    {
        let _write = state.write_lock.lock().await;
        sqlx::query(
            "INSERT INTO auth_states(state, nonce, code_verifier, expires_at) VALUES(?, ?, ?, ?)",
        )
        .bind(&state_token)
        .bind(&nonce)
        .bind(&verifier)
        .bind(expires_at as i64)
        .execute(&state.db)
        .await
        .map_err(internal)?;
        persist_state(&state).await?;
    }
    let mut url = Url::parse(&discovery.authorization_endpoint).map_err(|_| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "The sign-in service setup is not valid. Try again later.",
        )
    })?;
    url.query_pairs_mut()
        .append_pair("client_id", &auth.client_id)
        .append_pair("redirect_uri", &auth.redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("response_mode", "query")
        .append_pair("scope", "openid profile email")
        .append_pair("state", &state_token)
        .append_pair("nonce", &nonce)
        .append_pair("code_challenge", &pkce_challenge(&verifier))
        .append_pair("code_challenge_method", "S256");
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "{AUTH_STATE_COOKIE}={state_token}; Path=/api/auth/callback; Max-Age={AUTH_STATE_EXPIRY_SECONDS}; HttpOnly; Secure; SameSite=Lax"
        ))
        .map_err(internal)?,
    );
    Ok((
        headers,
        Json(AuthStart {
            authorization_url: url.into(),
        }),
    ))
}

async fn auth_callback(
    State(state): State<AppState>,
    Query(callback): Query<AuthCallback>,
    request: Request<Body>,
) -> Result<Response, ApiError> {
    let auth = state.auth.as_ref().ok_or(ApiError(
        StatusCode::SERVICE_UNAVAILABLE,
        "Sign-in is not configured for this service yet.",
    ))?;
    if callback.state.len() < 32 || callback.state.len() > 128 {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "This sign-in response is not valid. Start again from the account page.",
        ));
    }
    let browser_state = request_cookie(&request, AUTH_STATE_COOKIE).ok_or(ApiError(
        StatusCode::BAD_REQUEST,
        "This sign-in response does not match this browser. Start again from the account page.",
    ))?;
    if !tokens_equal(&browser_state, &callback.state) {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "This sign-in response does not match this browser. Start again from the account page.",
        ));
    }
    if callback.error.is_some() {
        let mut response = Redirect::to("/account?sign-in=cancelled").into_response();
        response.headers_mut().insert(
            header::SET_COOKIE,
            HeaderValue::from_str(&format!(
                "{AUTH_STATE_COOKIE}=; Path=/api/auth/callback; Max-Age=0; HttpOnly; Secure; SameSite=Lax"
            ))
            .map_err(internal)?,
        );
        return Ok(response);
    }
    let code = callback
        .code
        .filter(|value| !value.trim().is_empty())
        .ok_or(ApiError(
            StatusCode::BAD_REQUEST,
            "This sign-in response has no code. Start again from the account page.",
        ))?;
    let auth_state = {
        let _write = state.write_lock.lock().await;
        let row =
            sqlx::query("SELECT nonce, code_verifier, expires_at FROM auth_states WHERE state = ?")
                .bind(&callback.state)
                .fetch_optional(&state.db)
                .await
                .map_err(internal)?;
        sqlx::query("DELETE FROM auth_states WHERE state = ?")
            .bind(&callback.state)
            .execute(&state.db)
            .await
            .map_err(internal)?;
        persist_state(&state).await?;
        row
    }
    .ok_or(ApiError(
        StatusCode::GONE,
        "This sign-in request has expired. Start again from the account page.",
    ))?;
    let expires_at: i64 = auth_state.get("expires_at");
    if expires_at <= unix_now() as i64 {
        return Err(ApiError(
            StatusCode::GONE,
            "This sign-in request has expired. Start again from the account page.",
        ));
    }
    let nonce: String = auth_state.get("nonce");
    let code_verifier: String = auth_state.get("code_verifier");
    let discovery = oidc_discovery(&state.client, auth).await?;
    let tokens = state
        .client
        .post(&discovery.token_endpoint)
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", auth.client_id.as_str()),
            ("redirect_uri", auth.redirect_uri.as_str()),
            ("code", code.as_str()),
            ("code_verifier", code_verifier.as_str()),
        ])
        .send()
        .await
        .map_err(|_| {
            ApiError(
                StatusCode::BAD_GATEWAY,
                "The sign-in service could not finish. Start again.",
            )
        })?;
    if !tokens.status().is_success() {
        return Err(ApiError(
            StatusCode::UNAUTHORIZED,
            "The sign-in service did not accept this request. Start again.",
        ));
    }
    let tokens: OidcTokenResponse = tokens.json().await.map_err(|_| {
        ApiError(
            StatusCode::BAD_GATEWAY,
            "The sign-in service returned an unreadable result. Start again.",
        )
    })?;
    let user = verify_id_token(&state.client, auth, &discovery, &tokens.id_token, &nonce).await?;
    let session_token = random_url_token(32);
    let session_expires = unix_now() + SESSION_EXPIRY_SECONDS;
    {
        let _write = state.write_lock.lock().await;
        sqlx::query("INSERT INTO users(subject, display_name, email, created_at) VALUES(?, ?, ?, ?) ON CONFLICT(subject) DO UPDATE SET display_name=excluded.display_name, email=excluded.email")
            .bind(&user.subject)
            .bind(&user.display_name)
            .bind(&user.email)
            .bind(unix_now() as i64)
            .execute(&state.db)
            .await
            .map_err(internal)?;
        sqlx::query("DELETE FROM sessions WHERE expires_at <= ?")
            .bind(unix_now() as i64)
            .execute(&state.db)
            .await
            .map_err(internal)?;
        sqlx::query(
            "INSERT INTO sessions(token_hash, subject, expires_at, created_at) VALUES(?, ?, ?, ?)",
        )
        .bind(token_hash(&session_token))
        .bind(&user.subject)
        .bind(session_expires as i64)
        .bind(unix_now() as i64)
        .execute(&state.db)
        .await
        .map_err(internal)?;
        write_audit(
            &state,
            &format!("account:{}", user.subject),
            "account_signed_in",
            b"oidc_pkce",
        )
        .await?;
        persist_state(&state).await?;
    }
    let mut response = Redirect::to("/account?sign-in=complete").into_response();
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "{AUTH_STATE_COOKIE}=; Path=/api/auth/callback; Max-Age=0; HttpOnly; Secure; SameSite=Lax"
        ))
        .map_err(internal)?,
    );
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "{SESSION_COOKIE}={session_token}; Path=/; Max-Age={SESSION_EXPIRY_SECONDS}; HttpOnly; Secure; SameSite=Lax"
        ))
        .map_err(internal)?,
    );
    Ok(response)
}

async fn verify_id_token(
    client: &reqwest::Client,
    auth: &AuthConfig,
    discovery: &OidcDiscovery,
    id_token: &str,
    nonce: &str,
) -> Result<AuthenticatedUser, ApiError> {
    let token_header = decode_header(id_token).map_err(|_| {
        ApiError(
            StatusCode::UNAUTHORIZED,
            "The sign-in result is not valid. Start again.",
        )
    })?;
    if token_header.alg != Algorithm::RS256 {
        return Err(ApiError(
            StatusCode::UNAUTHORIZED,
            "The sign-in result uses an unsupported signature. Start again.",
        ));
    }
    let key_id = token_header.kid.ok_or(ApiError(
        StatusCode::UNAUTHORIZED,
        "The sign-in result has no signing key. Start again.",
    ))?;
    let jwks = client.get(&discovery.jwks_uri).send().await.map_err(|_| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "The sign-in service could not be verified. Start again.",
        )
    })?;
    if !jwks.status().is_success() {
        return Err(ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "The sign-in service could not be verified. Start again.",
        ));
    }
    let jwks: OidcJwks = jwks.json().await.map_err(|_| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "The sign-in service returned no usable signing key. Start again.",
        )
    })?;
    let key = jwks
        .keys
        .iter()
        .find(|key| key.kid.as_deref() == Some(key_id.as_str()) && key.kty == "RSA")
        .and_then(|key| Some((key.n.as_deref()?, key.e.as_deref()?)))
        .ok_or(ApiError(
            StatusCode::UNAUTHORIZED,
            "The sign-in result was signed with an unknown key. Start again.",
        ))?;
    let key = DecodingKey::from_rsa_components(key.0, key.1).map_err(|_| {
        ApiError(
            StatusCode::UNAUTHORIZED,
            "The sign-in signing key is not valid. Start again.",
        )
    })?;
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[auth.issuer.as_str()]);
    validation.set_audience(&[auth.client_id.as_str()]);
    let token = decode::<IdTokenClaims>(id_token, &key, &validation).map_err(|_| {
        ApiError(
            StatusCode::UNAUTHORIZED,
            "The sign-in result could not be verified. Start again.",
        )
    })?;
    let claims = token.claims;
    if claims.iss.trim_end_matches('/') != auth.issuer
        || claims.exp <= unix_now() as usize
        || claims.nonce.as_deref() != Some(nonce)
        || claims.sub.trim().is_empty()
        || claims.sub.len() > 512
    {
        return Err(ApiError(
            StatusCode::UNAUTHORIZED,
            "The sign-in result does not match this request. Start again.",
        ));
    }
    let display_name = claims
        .name
        .or(claims.preferred_username.clone())
        .or(claims.email.clone())
        .unwrap_or_else(|| "Quarterly Ready user".to_owned())
        .chars()
        .take(120)
        .collect();
    let email = claims
        .email
        .filter(|email| email.len() <= 320 && email.contains('@'));
    Ok(AuthenticatedUser {
        subject: format!("{}|{}", auth.issuer, claims.sub),
        display_name,
        email,
    })
}

async fn create_business(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<(StatusCode, Json<Business>), ApiError> {
    let user = authenticated_user(&state, request_cookie(&request, SESSION_COOKIE)).await?;
    let bytes = axum::body::to_bytes(request.into_body(), 16 * 1024)
        .await
        .map_err(|_| {
            ApiError(
                StatusCode::BAD_REQUEST,
                "The business details could not be read. Try again.",
            )
        })?;
    let input: CreateBusiness = serde_json::from_slice(&bytes).map_err(|_| {
        ApiError(
            StatusCode::BAD_REQUEST,
            "Enter a business name and try again.",
        )
    })?;
    let name = input.name.trim();
    if name.is_empty() || name.chars().count() > 120 {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Enter a business name of 120 characters or fewer.",
        ));
    }
    let business = Business {
        id: Uuid::new_v4().to_string(),
        name: name.to_owned(),
        role: "owner".to_owned(),
    };
    let _write = state.write_lock.lock().await;
    sqlx::query("INSERT INTO businesses(id, owner_subject, name, created_at) VALUES(?, ?, ?, ?)")
        .bind(&business.id)
        .bind(&user.subject)
        .bind(&business.name)
        .bind(unix_now() as i64)
        .execute(&state.db)
        .await
        .map_err(internal)?;
    sqlx::query(
        "INSERT INTO memberships(business_id, subject, role, created_at) VALUES(?, ?, 'owner', ?)",
    )
    .bind(&business.id)
    .bind(&user.subject)
    .bind(unix_now() as i64)
    .execute(&state.db)
    .await
    .map_err(internal)?;
    write_audit(
        &state,
        &format!("business:{}", business.id),
        "business_created",
        business.name.as_bytes(),
    )
    .await?;
    persist_state(&state).await?;
    Ok((StatusCode::CREATED, Json(business)))
}

fn account_quarter_key(business_id: &str, quarter_start: &str) -> String {
    format!("business:{business_id}:quarter:{quarter_start}")
}

fn validate_business_quarter(quarter_start: &str) -> Result<(), ApiError> {
    let period = quarter_from_start(quarter_start)
        .ok_or(ApiError(StatusCode::NOT_FOUND, "Quarter not found."))?;
    if period.0 != quarter_start {
        return Err(ApiError(StatusCode::NOT_FOUND, "Quarter not found."));
    }
    Ok(())
}

/// Return the matching standard-UK quarter end. Kept next to the authenticated
/// route checks so a path cannot point at a different reporting period.
fn quarter_from_start(start: &str) -> Option<(String, String)> {
    if !is_calendar_date(start) {
        return None;
    }
    let year = start.get(0..4)?.parse::<u32>().ok()?;
    let end = match start.get(5..)? {
        "04-06" => format!("{year:04}-07-05"),
        "07-06" => format!("{year:04}-10-05"),
        "10-06" => format!("{:04}-01-05", year + 1),
        "01-06" => format!("{year:04}-04-05"),
        _ => return None,
    };
    Some((start.to_owned(), end))
}

async fn stored_account_quarter(
    state: &AppState,
    business_id: &str,
    quarter_start: &str,
) -> Result<Option<Value>, ApiError> {
    let row = sqlx::query(
        "SELECT payload FROM account_quarters WHERE business_id = ? AND quarter_start = ?",
    )
    .bind(business_id)
    .bind(quarter_start)
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?;
    row.map(|row| decrypt_json(&state.key, &row.get::<Vec<u8>, _>("payload")))
        .transpose()
}

async fn get_account_quarter(
    State(state): State<AppState>,
    Path((business_id, quarter_start)): Path<(String, String)>,
    request: Request<Body>,
) -> Result<Json<AccountQuarterResult>, ApiError> {
    let user = authenticated_user(&state, request_cookie(&request, SESSION_COOKIE)).await?;
    validate_business_quarter(&quarter_start)?;
    require_business_membership(&state, &user.subject, &business_id).await?;
    Ok(Json(AccountQuarterResult {
        document: stored_account_quarter(&state, &business_id, &quarter_start).await?,
    }))
}

async fn put_account_quarter(
    State(state): State<AppState>,
    Path((business_id, quarter_start)): Path<(String, String)>,
    request: Request<Body>,
) -> Result<Json<AccountQuarterResult>, ApiError> {
    let user = authenticated_user(&state, request_cookie(&request, SESSION_COOKIE)).await?;
    validate_business_quarter(&quarter_start)?;
    let role = require_business_membership(&state, &user.subject, &business_id).await?;
    if role != "owner" {
        return Err(ApiError(
            StatusCode::FORBIDDEN,
            "This business membership cannot change records.",
        ));
    }
    let bytes = axum::body::to_bytes(request.into_body(), MAX_DOCUMENT_BYTES)
        .await
        .map_err(|_| {
            ApiError(
                StatusCode::BAD_REQUEST,
                "The records could not be read. Try saving again.",
            )
        })?;
    let input: AccountQuarterWrite = serde_json::from_slice(&bytes).map_err(|_| {
        ApiError(
            StatusCode::BAD_REQUEST,
            "The records are not valid JSON. Check the file and try again.",
        )
    })?;
    validate_document(&input.document)?;
    if input.document.get("quarterStart").and_then(Value::as_str) != Some(quarter_start.as_str()) {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "The records must belong to the selected quarter.",
        ));
    }
    let migration_id = input
        .migration_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty());
    if let Some(migration_id) = migration_id {
        if migration_id.len() > 128 || Uuid::parse_str(migration_id).is_err() {
            return Err(ApiError(
                StatusCode::UNPROCESSABLE_ENTITY,
                "The browser move could not be verified. Try it again from this device.",
            ));
        }
    }
    let encrypted = encrypt_json(&state.key, &input.document)?;
    let _write = state.write_lock.lock().await;
    if let Some(migration_id) = migration_id {
        let existing = sqlx::query("SELECT 1 FROM account_migrations WHERE business_id = ? AND quarter_start = ? AND migration_id = ?")
            .bind(&business_id)
            .bind(&quarter_start)
            .bind(migration_id)
            .fetch_optional(&state.db)
            .await
            .map_err(internal)?;
        if existing.is_some() {
            return Ok(Json(AccountQuarterResult {
                document: stored_account_quarter(&state, &business_id, &quarter_start).await?,
            }));
        }
    }
    sqlx::query("INSERT INTO account_quarters(business_id, quarter_start, payload, updated_at) VALUES(?, ?, ?, ?) ON CONFLICT(business_id, quarter_start) DO UPDATE SET payload=excluded.payload, updated_at=excluded.updated_at")
        .bind(&business_id)
        .bind(&quarter_start)
        .bind(encrypted)
        .bind(unix_now() as i64)
        .execute(&state.db)
        .await
        .map_err(internal)?;
    let audit_key = account_quarter_key(&business_id, &quarter_start);
    let action = if migration_id.is_some() {
        "browser_quarter_migrated"
    } else {
        "account_quarter_saved"
    };
    write_audit(&state, &audit_key, action, &bytes).await?;
    if let Some(migration_id) = migration_id {
        sqlx::query("INSERT INTO account_migrations(business_id, quarter_start, migration_id, created_at) VALUES(?, ?, ?, ?)")
            .bind(&business_id)
            .bind(&quarter_start)
            .bind(migration_id)
            .bind(unix_now() as i64)
            .execute(&state.db)
            .await
            .map_err(internal)?;
    }
    persist_state(&state).await?;
    Ok(Json(AccountQuarterResult {
        document: Some(input.document),
    }))
}

async fn create_account_share(
    State(state): State<AppState>,
    Path((business_id, quarter_start)): Path<(String, String)>,
    request: Request<Body>,
) -> Result<(StatusCode, Json<ShareResult>), ApiError> {
    let user = authenticated_user(&state, request_cookie(&request, SESSION_COOKIE)).await?;
    validate_business_quarter(&quarter_start)?;
    let role = require_business_membership(&state, &user.subject, &business_id).await?;
    if role != "owner" {
        return Err(ApiError(
            StatusCode::FORBIDDEN,
            "This business membership cannot create an accountant link.",
        ));
    }
    let licence = licence_token(&request)?;
    verify_licence_token(&state, &licence).await?;
    let document = stored_account_quarter(&state, &business_id, &quarter_start)
        .await?
        .ok_or(ApiError(
            StatusCode::NOT_FOUND,
            "Save this quarter to your account before making an accountant link.",
        ))?;
    let token = Uuid::new_v4().simple().to_string();
    let expires_at = unix_now() + SHARE_EXPIRY_SECONDS;
    let encrypted = encrypt_json(&state.key, &document)?;
    let _write = state.write_lock.lock().await;
    sqlx::query("INSERT INTO account_shares(token, business_id, quarter_start, payload, expires_at) VALUES(?, ?, ?, ?, ?)")
        .bind(&token)
        .bind(&business_id)
        .bind(&quarter_start)
        .bind(encrypted)
        .bind(expires_at as i64)
        .execute(&state.db)
        .await
        .map_err(internal)?;
    write_audit(
        &state,
        &account_quarter_key(&business_id, &quarter_start),
        "accountant_link_created",
        token.as_bytes(),
    )
    .await?;
    persist_state(&state).await?;
    Ok((StatusCode::CREATED, Json(ShareResult { token, expires_at })))
}

async fn export_account(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<Json<Value>, ApiError> {
    let user = authenticated_user(&state, request_cookie(&request, SESSION_COOKIE)).await?;
    let businesses = businesses_for_user(&state, &user.subject).await?;
    let mut exported = Vec::new();
    for business in &businesses {
        let rows = sqlx::query("SELECT quarter_start, payload, updated_at FROM account_quarters WHERE business_id = ? ORDER BY quarter_start")
            .bind(&business.id)
            .fetch_all(&state.db)
            .await
            .map_err(internal)?;
        let quarters = rows
            .into_iter()
            .map(|row| {
                Ok(json!({
                    "quarter_start": row.get::<String, _>("quarter_start"),
                    "updated_at": row.get::<i64, _>("updated_at"),
                    "document": decrypt_json(&state.key, &row.get::<Vec<u8>, _>("payload"))?,
                }))
            })
            .collect::<Result<Vec<_>, ApiError>>()?;
        exported.push(json!({ "business": business, "quarters": quarters }));
    }
    Ok(Json(json!({
        "format": "quarterly-ready-account-export-v1",
        "created_at": unix_now(),
        "account": { "display_name": user.display_name, "email": user.email },
        "businesses": exported,
    })))
}

async fn delete_account(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<StatusCode, ApiError> {
    let user = authenticated_user(&state, request_cookie(&request, SESSION_COOKIE)).await?;
    let _write = state.write_lock.lock().await;
    let owned = sqlx::query("SELECT id FROM businesses WHERE owner_subject = ?")
        .bind(&user.subject)
        .fetch_all(&state.db)
        .await
        .map_err(internal)?;
    for row in owned {
        let business_id: String = row.get("id");
        sqlx::query("DELETE FROM account_shares WHERE business_id = ?")
            .bind(&business_id)
            .execute(&state.db)
            .await
            .map_err(internal)?;
        sqlx::query("DELETE FROM account_migrations WHERE business_id = ?")
            .bind(&business_id)
            .execute(&state.db)
            .await
            .map_err(internal)?;
        sqlx::query("DELETE FROM account_quarters WHERE business_id = ?")
            .bind(&business_id)
            .execute(&state.db)
            .await
            .map_err(internal)?;
        sqlx::query("DELETE FROM memberships WHERE business_id = ?")
            .bind(&business_id)
            .execute(&state.db)
            .await
            .map_err(internal)?;
        sqlx::query("DELETE FROM businesses WHERE id = ?")
            .bind(&business_id)
            .execute(&state.db)
            .await
            .map_err(internal)?;
    }
    sqlx::query("DELETE FROM memberships WHERE subject = ?")
        .bind(&user.subject)
        .execute(&state.db)
        .await
        .map_err(internal)?;
    sqlx::query("DELETE FROM sessions WHERE subject = ?")
        .bind(&user.subject)
        .execute(&state.db)
        .await
        .map_err(internal)?;
    sqlx::query("DELETE FROM users WHERE subject = ?")
        .bind(&user.subject)
        .execute(&state.db)
        .await
        .map_err(internal)?;
    write_audit(
        &state,
        &format!("account:{}", user.subject),
        "account_deleted",
        b"self_service",
    )
    .await?;
    persist_state(&state).await?;
    Ok(StatusCode::NO_CONTENT)
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
        return Ok(Json(json!({ "document": null })));
    };
    let payload: Vec<u8> = row.get("payload");
    let document = decrypt_json(&state.key, &payload)?;
    Ok(Json(json!({ "document": document })))
}

async fn put_workspace(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<Json<Value>, ApiError> {
    let _write = state.write_lock.lock().await;
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
    let mut transaction = state.db.begin().await.map_err(internal)?;
    sqlx::query("INSERT INTO workspaces(id, payload, updated_at) VALUES(?, ?, ?) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload, updated_at=excluded.updated_at")
        .bind(&id).bind(encrypted).bind(now as i64).execute(&mut *transaction).await.map_err(internal)?;
    write_audit_transaction(&mut transaction, &id, "records_saved", &bytes).await?;
    transaction.commit().await.map_err(internal)?;
    persist_state(&state).await?;
    Ok(Json(json!({ "saved": true, "updated_at": now })))
}

async fn create_share(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<(StatusCode, Json<ShareResult>), ApiError> {
    let _write = state.write_lock.lock().await;
    let id = workspace_id(&request)?;
    let licence = licence_token(&request)?;
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
    if !safe_fixture_authorized(state.safe_qa_fixtures, &licence, &input.document) {
        verify_licence_token(&state, &licence).await?;
    }
    let token = Uuid::new_v4().simple().to_string();
    let expires_at = unix_now() + SHARE_EXPIRY_SECONDS;
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
    persist_state(&state).await?;
    Ok((StatusCode::CREATED, Json(ShareResult { token, expires_at })))
}

async fn submit_to_hmrc(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<Json<SubmissionResult>, ApiError> {
    let _write = state.write_lock.lock().await;
    let id = workspace_id(&request)?;
    let licence = request
        .headers()
        .get("x-sociobot-license")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|token| !token.is_empty() && token.len() <= 2048)
        .map(str::to_owned);
    let bytes = axum::body::to_bytes(request.into_body(), MAX_DOCUMENT_BYTES)
        .await
        .map_err(|_| {
            ApiError(
                StatusCode::BAD_REQUEST,
                "The submission could not be read. Try again.",
            )
        })?;
    let input: SubmissionRequest = serde_json::from_slice(&bytes).map_err(|_| {
        ApiError(
            StatusCode::BAD_REQUEST,
            "The submission is not valid. Review the quarter and try again.",
        )
    })?;
    if !input.review_confirmed {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Confirm that you reviewed the totals before submitting to HMRC.",
        ));
    }
    let payload = hmrc_compatible_payload(&input.document, input.review_confirmed)?;
    let licence = licence.ok_or(ApiError(
        StatusCode::PAYMENT_REQUIRED,
        "An active Sociobot subscription is required for live accountant links and HMRC submissions.",
    ))?;
    let safe_fixture = safe_fixture_authorized(state.safe_qa_fixtures, &licence, &input.document);
    if safe_fixture
        && state
            .hmrc_integration
            .as_ref()
            .is_none_or(|integration| integration.mode != IntegrationMode::HmrcSandboxNoFiling)
    {
        let submission_id = format!("safe-fixture-no-filing-{}", unix_now());
        write_audit(
            &state,
            &id,
            "safe_fixture_submission_checked",
            submission_id.as_bytes(),
        )
        .await?;
        persist_state(&state).await?;
        return Ok(Json(SubmissionResult {
            submission_id,
            status: "fixture_only_no_filing",
            files_with_hmrc: false,
        }));
    }
    if !safe_fixture {
        verify_licence_token(&state, &licence).await?;
    }
    let integration = state.hmrc_integration.as_ref().ok_or(ApiError(
        StatusCode::SERVICE_UNAVAILABLE,
        "An approved HMRC integration is not configured for this service. Download the accountant pack or try again later.",
    ))?;
    let taxpayer_access_token = if integration.taxpayer_consent.is_some() {
        Some(load_taxpayer_consent_token(&state, &id).await?)
    } else {
        None
    };
    let result = send_to_approved_integration(
        &state.client,
        integration,
        &payload,
        taxpayer_access_token.as_deref(),
    )
    .await?;
    write_audit(
        &state,
        &id,
        if result.files_with_hmrc {
            "hmrc_submission_requested"
        } else {
            "hmrc_sandbox_validation_completed"
        },
        result.submission_id.as_bytes(),
    )
    .await?;
    persist_state(&state).await?;
    Ok(Json(result))
}

async fn send_to_approved_integration(
    client: &reqwest::Client,
    integration: &ApprovedIntegration,
    payload: &Value,
    taxpayer_access_token: Option<&str>,
) -> Result<SubmissionResult, ApiError> {
    if integration.mode == IntegrationMode::HmrcSandboxNoFiling {
        let response = client
            .get(&integration.url)
            .header("accept", "application/vnd.hmrc.1.0+json")
            .send()
            .await
            .map_err(|_| {
                ApiError(
                    StatusCode::BAD_GATEWAY,
                    "The HMRC test API could not be reached. No submission was made.",
                )
            })?;
        if !response.status().is_success() {
            return Err(ApiError(
                StatusCode::BAD_GATEWAY,
                "The HMRC test API rejected the sandbox check. No submission was made.",
            ));
        }
        let response: Value = response.json().await.map_err(|_| {
            ApiError(
                StatusCode::BAD_GATEWAY,
                "The HMRC test API returned an unreadable sandbox response. No submission was made.",
            )
        })?;
        if response.get("message").and_then(Value::as_str) != Some("Hello World") {
            return Err(ApiError(
                StatusCode::BAD_GATEWAY,
                "The HMRC test API did not confirm the sandbox check. No submission was made.",
            ));
        }
        let mut proof = Sha256::new();
        proof.update(integration.token.as_bytes());
        proof.update(serde_json::to_vec(payload).map_err(internal)?);
        proof.update(unix_now().to_be_bytes());
        let proof = format!("{:x}", proof.finalize());
        return Ok(SubmissionResult {
            submission_id: format!("hmrc-sandbox-no-filing-{}", &proof[..16]),
            status: "sandbox_accepted_no_filing",
            files_with_hmrc: false,
        });
    }

    let taxpayer_access_token = taxpayer_access_token.ok_or(ApiError(
        StatusCode::PRECONDITION_REQUIRED,
        "Connect and authorise your tax account before submitting a quarterly update.",
    ))?;
    let response = client
        .post(&integration.url)
        .bearer_auth(taxpayer_access_token)
        .header("x-quarterly-ready-provider-token", &integration.token)
        .header(
            "x-quarterly-ready-submission",
            "mtd-itsa-periodic-update-v1",
        )
        .json(payload)
        .send()
        .await
        .map_err(|_| {
            ApiError(
                StatusCode::BAD_GATEWAY,
                "The approved HMRC integration could not be reached. No submission was made.",
            )
        })?;
    if !response.status().is_success() {
        return Err(ApiError(
            StatusCode::BAD_GATEWAY,
            "The approved HMRC integration rejected the submission. No submission was made.",
        ));
    }
    let response: Value = response.json().await.map_err(|_| {
        ApiError(
            StatusCode::BAD_GATEWAY,
            "The approved HMRC integration did not return a submission reference. No submission was made.",
        )
    })?;
    let submission_id = response
        .get("submission_id")
        .or_else(|| response.get("correlation_id"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or(ApiError(
            StatusCode::BAD_GATEWAY,
            "The approved HMRC integration did not return a submission reference. No submission was made.",
        ))?
        .to_owned();
    Ok(SubmissionResult {
        submission_id,
        status: "accepted",
        files_with_hmrc: true,
    })
}

async fn hmrc_consent_status(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<Json<ConsentStatus>, ApiError> {
    let id = workspace_id(&request)?;
    if state
        .hmrc_integration
        .as_ref()
        .is_none_or(|integration| integration.taxpayer_consent.is_none())
    {
        return Ok(Json(ConsentStatus {
            consented: false,
            expires_at: None,
        }));
    }
    let row = sqlx::query("SELECT expires_at FROM hmrc_consents WHERE workspace_id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal)?;
    let expires_at = row.map(|row| row.get::<i64, _>("expires_at") as u64);
    Ok(Json(ConsentStatus {
        consented: expires_at.is_some_and(|expiry| expiry > unix_now()),
        expires_at: expires_at.filter(|expiry| *expiry > unix_now()),
    }))
}

async fn start_hmrc_consent(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<Json<ConsentStart>, ApiError> {
    let id = workspace_id(&request)?;
    let consent = state
        .hmrc_integration
        .as_ref()
        .and_then(|integration| integration.taxpayer_consent.as_ref())
        .ok_or(ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "Taxpayer consent is unavailable because no approved HMRC provider is configured.",
        ))?
        .clone();
    let state_token = Uuid::new_v4().simple().to_string();
    let expires_at = unix_now() + 10 * 60;
    let _write = state.write_lock.lock().await;
    sqlx::query("INSERT INTO hmrc_consent_states(state, workspace_id, expires_at) VALUES(?, ?, ?) ON CONFLICT(state) DO UPDATE SET workspace_id=excluded.workspace_id, expires_at=excluded.expires_at")
        .bind(&state_token)
        .bind(&id)
        .bind(expires_at as i64)
        .execute(&state.db)
        .await
        .map_err(internal)?;
    write_audit(
        &state,
        &id,
        "taxpayer_consent_started",
        consent.provider_approval_reference.as_bytes(),
    )
    .await?;
    persist_state(&state).await?;
    let mut url = Url::parse(&consent.authorize_url).map_err(|_| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "The configured HMRC provider consent URL is not valid.",
        )
    })?;
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", &consent.client_id)
        .append_pair("redirect_uri", &consent.redirect_uri)
        .append_pair("state", &state_token)
        .append_pair("scope", "write:self-assessment");
    Ok(Json(ConsentStart {
        authorization_url: url.into(),
    }))
}

async fn hmrc_consent_callback(
    State(state): State<AppState>,
    Query(callback): Query<ConsentCallback>,
) -> Result<Redirect, ApiError> {
    if callback.state.len() != 32
        || !callback
            .state
            .chars()
            .all(|value| value.is_ascii_hexdigit())
    {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "The taxpayer consent response is not valid. Start again from your quarter.",
        ));
    }
    if callback.error.is_some() {
        return Err(ApiError(
            StatusCode::UNAUTHORIZED,
            "Taxpayer consent was not granted. No quarterly update can be submitted.",
        ));
    }
    let code = callback
        .code
        .filter(|value| !value.trim().is_empty())
        .ok_or(ApiError(
            StatusCode::BAD_REQUEST,
            "The taxpayer consent response did not include an authorisation code.",
        ))?;
    let consent = state
        .hmrc_integration
        .as_ref()
        .and_then(|integration| integration.taxpayer_consent.as_ref())
        .ok_or(ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "Taxpayer consent is unavailable because no approved HMRC provider is configured.",
        ))?
        .clone();
    let state_row = {
        let _write = state.write_lock.lock().await;
        let row =
            sqlx::query("SELECT workspace_id, expires_at FROM hmrc_consent_states WHERE state = ?")
                .bind(&callback.state)
                .fetch_optional(&state.db)
                .await
                .map_err(internal)?;
        sqlx::query("DELETE FROM hmrc_consent_states WHERE state = ?")
            .bind(&callback.state)
            .execute(&state.db)
            .await
            .map_err(internal)?;
        persist_state(&state).await?;
        row
    };
    let state_row = state_row.ok_or(ApiError(
        StatusCode::GONE,
        "This taxpayer consent request has expired. Start again from your quarter.",
    ))?;
    let workspace_id: String = state_row.get("workspace_id");
    let state_expires_at: i64 = state_row.get("expires_at");
    if state_expires_at < unix_now() as i64 {
        return Err(ApiError(
            StatusCode::GONE,
            "This taxpayer consent request has expired. Start again from your quarter.",
        ));
    }
    let response = state
        .client
        .post(&consent.token_url)
        .basic_auth(&consent.client_id, Some(&consent.client_secret))
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code.as_str()),
            ("redirect_uri", consent.redirect_uri.as_str()),
        ])
        .send()
        .await
        .map_err(|_| {
            ApiError(
                StatusCode::BAD_GATEWAY,
                "The approved HMRC provider could not complete taxpayer consent. No submission was made.",
            )
        })?;
    if !response.status().is_success() {
        return Err(ApiError(
            StatusCode::BAD_GATEWAY,
            "The approved HMRC provider rejected taxpayer consent. No submission was made.",
        ));
    }
    let token: ProviderTokenResponse = response.json().await.map_err(|_| {
        ApiError(
            StatusCode::BAD_GATEWAY,
            "The approved HMRC provider returned an unreadable taxpayer consent result.",
        )
    })?;
    if token.access_token.trim().is_empty() {
        return Err(ApiError(
            StatusCode::BAD_GATEWAY,
            "The approved HMRC provider did not return taxpayer consent credentials.",
        ));
    }
    let expires_at = unix_now() + token.expires_in.unwrap_or(3600).clamp(60, 86_400);
    let encrypted = encrypt_json(
        &state.key,
        &json!({ "access_token": token.access_token, "refresh_token": token.refresh_token }),
    )?;
    let _write = state.write_lock.lock().await;
    sqlx::query("INSERT INTO hmrc_consents(workspace_id, payload, expires_at, created_at) VALUES(?, ?, ?, ?) ON CONFLICT(workspace_id) DO UPDATE SET payload=excluded.payload, expires_at=excluded.expires_at, created_at=excluded.created_at")
        .bind(&workspace_id)
        .bind(encrypted)
        .bind(expires_at as i64)
        .bind(unix_now() as i64)
        .execute(&state.db)
        .await
        .map_err(internal)?;
    write_audit(
        &state,
        &workspace_id,
        "taxpayer_consent_granted",
        b"provider_oauth",
    )
    .await?;
    persist_state(&state).await?;
    Ok(Redirect::to("/records?hmrc-consent=connected"))
}

async fn load_taxpayer_consent_token(
    state: &AppState,
    workspace_id: &str,
) -> Result<String, ApiError> {
    let row = sqlx::query("SELECT payload, expires_at FROM hmrc_consents WHERE workspace_id = ?")
        .bind(workspace_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal)?;
    let row = row.ok_or(ApiError(
        StatusCode::PRECONDITION_REQUIRED,
        "Connect and authorise your tax account before submitting a quarterly update.",
    ))?;
    let expires_at: i64 = row.get("expires_at");
    if expires_at <= unix_now() as i64 {
        return Err(ApiError(
            StatusCode::PRECONDITION_REQUIRED,
            "Your taxpayer consent has expired. Connect your tax account again before submitting.",
        ));
    }
    let payload: Vec<u8> = row.get("payload");
    decrypt_json(&state.key, &payload)?
        .get("access_token")
        .and_then(Value::as_str)
        .filter(|token| !token.trim().is_empty())
        .map(str::to_owned)
        .ok_or(ApiError(
            StatusCode::INTERNAL_SERVER_ERROR,
            "The saved taxpayer consent is incomplete. Connect your tax account again.",
        ))
}

async fn safe_qa_entitlement(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    if !state.safe_qa_fixtures {
        return Err(ApiError(
            StatusCode::NOT_FOUND,
            "The safe QA fixture is not enabled.",
        ));
    }
    Ok(Json(json!({
        "token": SAFE_FIXTURE_TOKEN,
        "charges": false,
        "files_with_hmrc": false,
        "document": safe_fixture_document()
    })))
}

fn safe_fixtures_enabled() -> bool {
    env::var("SAFE_QA_FIXTURES").is_ok_and(|value| value == "1")
}

fn safe_fixture_authorized(enabled: bool, token: &str, document: &Value) -> bool {
    enabled && token == SAFE_FIXTURE_TOKEN && document == &safe_fixture_document()
}

fn safe_fixture_document() -> Value {
    json!({
        "schemaVersion": 1,
        "businessName": SAFE_FIXTURE_BUSINESS,
        "quarterLabel": "6 July to 5 October 2099",
        "quarterStart": "2099-07-06",
        "quarterEnd": "2099-10-05",
        "figuresReviewed": true,
        "packDownloaded": true,
        "markedReady": true,
        "updatedAt": "2099-08-29T12:00:00.000Z",
        "transactions": [{
            "id": "safe-fixture-income-1",
            "date": "2099-08-29",
            "description": "Synthetic QA income — no customer data",
            "amountPence": 100,
            "kind": "income",
            "category": "Sales"
        }]
    })
}

fn licence_token(request: &Request<Body>) -> Result<String, ApiError> {
    request
        .headers()
        .get("x-sociobot-license")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|token| !token.is_empty() && token.len() <= 2048)
        .map(str::to_owned)
        .ok_or(ApiError(
            StatusCode::PAYMENT_REQUIRED,
            "An active Sociobot subscription is required for live accountant links and HMRC submissions.",
        ))
}

async fn verify_licence_token(state: &AppState, token: &str) -> Result<(), ApiError> {
    let mut reached_service = false;
    for slug in PRODUCT_SLUGS {
        let endpoint = format!(
            "{}/products/{slug}/verify",
            state.billing_base_url.trim_end_matches('/')
        );
        let response = match state
            .client
            .get(endpoint)
            .query(&[("license", token)])
            .send()
            .await
        {
            Ok(response) => response,
            Err(_) => continue,
        };
        reached_service = true;
        if !response.status().is_success() {
            continue;
        }
        let verdict: LicenceVerdict = response.json().await.map_err(|_| ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "The Sociobot licence service gave an unreadable response. Try again before creating a live link.",
        ))?;
        if verdict.valid {
            return Ok(());
        }
    }
    if !reached_service {
        return Err(ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "The Sociobot licence service could not be reached. Try again before creating a live link.",
        ));
    }
    Err(ApiError(
        StatusCode::PAYMENT_REQUIRED,
        "Your Sociobot subscription is not active. Check the licence or choose a subscription.",
    ))
}

fn hmrc_compatible_payload(document: &Value, review_confirmed: bool) -> Result<Value, ApiError> {
    validate_document(document)?;
    if !review_confirmed {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Confirm that you reviewed the totals before submitting to HMRC.",
        ));
    }
    let object = document.as_object().expect("validated document object");
    if object.get("figuresReviewed").and_then(Value::as_bool) != Some(true)
        || object.get("markedReady").and_then(Value::as_bool) != Some(true)
    {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Complete the quarter checklist and mark the quarter ready before submitting.",
        ));
    }
    let period_start = object
        .get("quarterStart")
        .and_then(Value::as_str)
        .filter(|value| is_iso_date(value))
        .ok_or(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "The quarter needs a valid start date before it can be submitted.",
        ))?;
    let period_end = object
        .get("quarterEnd")
        .and_then(Value::as_str)
        .filter(|value| is_iso_date(value))
        .ok_or(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "The quarter needs a valid end date before it can be submitted.",
        ))?;
    let transactions = object
        .get("transactions")
        .and_then(Value::as_array)
        .expect("validated transaction list");
    if transactions.is_empty() {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Add at least one transaction before submitting.",
        ));
    }
    let mut turnover_pence = 0_i64;
    let mut expenses: HashMap<String, i64> = HashMap::new();
    for transaction in transactions {
        let entry = transaction.as_object().ok_or(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Each transaction must be a record.",
        ))?;
        let amount = entry
            .get("amountPence")
            .and_then(Value::as_i64)
            .filter(|amount| *amount > 0)
            .ok_or(ApiError(
                StatusCode::UNPROCESSABLE_ENTITY,
                "Every transaction needs an amount greater than zero before submission.",
            ))?;
        let kind = entry.get("kind").and_then(Value::as_str).ok_or(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Every transaction needs a type before submission.",
        ))?;
        let category = entry
            .get("category")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|category| !category.is_empty())
            .ok_or(ApiError(
                StatusCode::UNPROCESSABLE_ENTITY,
                "Resolve every transaction category before submitting.",
            ))?;
        match kind {
            "income" => turnover_pence += amount,
            "expense" => {
                if entry
                    .get("receiptName")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|name| !name.is_empty())
                    .is_none()
                {
                    return Err(ApiError(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "Attach or check every expense receipt before submitting.",
                    ));
                }
                *expenses
                    .entry(category.to_lowercase().replace(' ', "_"))
                    .or_default() += amount;
            }
            _ => {
                return Err(ApiError(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "Every transaction needs an income or expense type before submission.",
                ))
            }
        }
    }
    let period_expenses = expenses
        .into_iter()
        .map(|(category, pence)| (category, Value::from(pence as f64 / 100.0)))
        .collect::<serde_json::Map<_, _>>();
    Ok(json!({
        "format": "quarterly-ready-mtd-itsa-periodic-update-v1",
        "periodStartDate": period_start,
        "periodEndDate": period_end,
        "periodIncome": { "turnover": turnover_pence as f64 / 100.0 },
        "periodExpenses": period_expenses,
        "reviewedByUser": true,
        "humanReviewConfirmedAt": unix_now(),
    }))
}

fn is_iso_date(value: &str) -> bool {
    value.len() == 10
        && value.as_bytes().get(4) == Some(&b'-')
        && value.as_bytes().get(7) == Some(&b'-')
        && value
            .bytes()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
}

fn approved_integration_from_env() -> Option<ApprovedIntegration> {
    approved_integration_from_values(
        env::var("HMRC_INTEGRATION_URL").ok(),
        env::var("HMRC_INTEGRATION_TOKEN").ok(),
        env::var("HMRC_INTEGRATION_MODE").ok(),
        taxpayer_consent_from_env(),
    )
}

fn taxpayer_consent_from_env() -> Option<TaxpayerConsent> {
    taxpayer_consent_from_values(
        env::var("HMRC_CONSENT_AUTHORIZE_URL").ok(),
        env::var("HMRC_CONSENT_TOKEN_URL").ok(),
        env::var("HMRC_CONSENT_CLIENT_ID").ok(),
        env::var("HMRC_CONSENT_CLIENT_SECRET").ok(),
        env::var("HMRC_CONSENT_REDIRECT_URI").ok(),
        env::var("HMRC_PROVIDER_NAME").ok(),
        env::var("HMRC_PROVIDER_APPROVAL_REFERENCE").ok(),
    )
}

fn taxpayer_consent_from_values(
    authorize_url: Option<String>,
    token_url: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
    redirect_uri: Option<String>,
    provider_name: Option<String>,
    provider_approval_reference: Option<String>,
) -> Option<TaxpayerConsent> {
    let authorize_url = authorize_url?;
    let token_url = token_url?;
    let client_id = client_id?;
    let client_secret = client_secret?;
    let redirect_uri = redirect_uri?;
    let provider_name = provider_name?;
    let provider_approval_reference = provider_approval_reference?;
    let non_empty = [
        &client_id,
        &client_secret,
        &provider_name,
        &provider_approval_reference,
    ]
    .iter()
    .all(|value| !value.trim().is_empty());
    if !non_empty
        || !authorize_url.starts_with("https://")
        || !token_url.starts_with("https://")
        || !redirect_uri.starts_with("https://")
    {
        return None;
    }
    Some(TaxpayerConsent {
        authorize_url,
        token_url,
        client_id,
        client_secret,
        redirect_uri,
        provider_name,
        provider_approval_reference,
    })
}

fn approved_integration_from_values(
    url: Option<String>,
    token: Option<String>,
    mode: Option<String>,
    taxpayer_consent: Option<TaxpayerConsent>,
) -> Option<ApprovedIntegration> {
    let url = url?;
    let token = token?;
    if !url.starts_with("https://") || token.trim().is_empty() {
        return None;
    }
    let mode = match mode.as_deref() {
        Some("hmrc_sandbox_no_filing")
            if url == "https://test-api.service.hmrc.gov.uk/hello/world" =>
        {
            IntegrationMode::HmrcSandboxNoFiling
        }
        None | Some("") | Some("approved_provider")
            if taxpayer_consent.is_some()
                && url != "https://test-api.service.hmrc.gov.uk/hello/world" =>
        {
            IntegrationMode::ApprovedProvider
        }
        _ => return None,
    };
    Some(ApprovedIntegration {
        url,
        token,
        mode,
        taxpayer_consent: if mode == IntegrationMode::ApprovedProvider {
            taxpayer_consent
        } else {
            None
        },
    })
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
    let row = sqlx::query("SELECT payload, expires_at FROM account_shares WHERE token = ? UNION ALL SELECT payload, expires_at FROM shares WHERE token = ? LIMIT 1")
        .bind(&token)
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
    let _write = state.write_lock.lock().await;
    let day = unix_now() / 86_400;
    sqlx::query("INSERT INTO page_views(day, count) VALUES(?, 1) ON CONFLICT(day) DO UPDATE SET count=count+1")
        .bind(day as i64).execute(&state.db).await.map_err(internal)?;
    persist_state(&state).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn security_headers(request: Request<Body>, next: Next) -> Response {
    let immutable_asset = request.uri().path().starts_with("/assets/");
    let revalidate = request.uri().path() == "/sw.js"
        || !request.uri().path().contains('.')
        || request.uri().path().ends_with(".html");
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
    headers.insert(
        HeaderName::from_static("strict-transport-security"),
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    headers.insert(HeaderName::from_static("content-security-policy"), HeaderValue::from_static("default-src 'self'; img-src 'self' data: blob:; style-src 'self'; script-src 'self'; connect-src 'self' https://api.sociobot.in; object-src 'none'; base-uri 'self'; form-action 'self' https://api.sociobot.in; frame-ancestors 'none'"));
    if immutable_asset {
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=31536000, immutable"),
        );
    } else if revalidate {
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
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
    let quarter_start = object
        .get("quarterStart")
        .and_then(Value::as_str)
        .filter(|value| is_calendar_date(value))
        .ok_or(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "The quarter needs a valid start date.",
        ))?;
    let quarter_end = object
        .get("quarterEnd")
        .and_then(Value::as_str)
        .filter(|value| is_calendar_date(value))
        .ok_or(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "The quarter needs a valid end date.",
        ))?;
    if !is_standard_uk_quarter(quarter_start, quarter_end) {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Choose a standard UK quarter with matching start and end dates.",
        ));
    }
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
    let mut ids = std::collections::HashSet::with_capacity(transactions.len());
    for transaction in transactions {
        validate_transaction(transaction, &mut ids, quarter_start, quarter_end)?;
    }
    Ok(())
}

fn invalid_transaction(message: &'static str) -> ApiError {
    ApiError(StatusCode::UNPROCESSABLE_ENTITY, message)
}

fn validate_transaction(
    transaction: &Value,
    ids: &mut std::collections::HashSet<String>,
    quarter_start: &str,
    quarter_end: &str,
) -> Result<(), ApiError> {
    let object = transaction
        .as_object()
        .ok_or_else(|| invalid_transaction("Every transaction must be an object."))?;
    let id = object
        .get("id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.len() <= 128)
        .ok_or_else(|| invalid_transaction("Every transaction needs an ID."))?;
    if !ids.insert(id.to_owned()) {
        return Err(invalid_transaction("Transaction IDs must be unique."));
    }
    let date = object
        .get("date")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| is_calendar_date(value))
        .ok_or_else(|| invalid_transaction("Every transaction needs a valid date."))?;
    if date < quarter_start || date > quarter_end {
        return Err(invalid_transaction(
            "Every transaction date must be inside the selected quarter.",
        ));
    }
    object
        .get("description")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.chars().count() <= 120)
        .ok_or_else(|| {
            invalid_transaction("Every transaction needs a description of 120 characters or fewer.")
        })?;
    object
        .get("amountPence")
        .and_then(Value::as_i64)
        .filter(|value| (1..=100_000_000).contains(value))
        .ok_or_else(|| {
            invalid_transaction("Every transaction needs an amount between £0.01 and £1,000,000.")
        })?;
    match object.get("kind").and_then(Value::as_str) {
        Some("income" | "expense") => {}
        _ => {
            return Err(invalid_transaction(
                "Every transaction needs an income or expense type.",
            ))
        }
    }
    match object.get("category").and_then(Value::as_str) {
        Some("") => {}
        Some(value) if CATEGORIES.contains(&value) => {}
        _ => {
            return Err(invalid_transaction(
                "Every transaction needs a recognised category or an empty category for review.",
            ))
        }
    }
    let receipt_name = match object.get("receiptName") {
        None => None,
        Some(Value::String(value)) if !value.trim().is_empty() && value.chars().count() <= 255 => {
            Some(value)
        }
        _ => {
            return Err(invalid_transaction(
                "A receipt name must be 255 characters or fewer.",
            ))
        }
    };
    if let Some(data) = object.get("receiptData") {
        let data = data
            .as_str()
            .filter(|value| {
                value.starts_with("data:image/jpeg;base64,")
                    || value.starts_with("data:image/png;base64,")
                    || value.starts_with("data:application/pdf;base64,")
            })
            .filter(|value| value.len() <= 2_000_000)
            .ok_or_else(|| {
                invalid_transaction("Receipt data must be a JPEG, PNG, or PDF under 1.5 MB.")
            })?;
        let _ = data;
        if receipt_name.is_none() {
            return Err(invalid_transaction("Receipt data needs a receipt name."));
        }
    }
    if let Some(note) = object.get("note") {
        if note
            .as_str()
            .is_none_or(|value| value.chars().count() > 1_000)
        {
            return Err(invalid_transaction(
                "A transaction note must be text of 1,000 characters or fewer.",
            ));
        }
    }
    Ok(())
}

fn is_calendar_date(value: &str) -> bool {
    if !is_iso_date(value) {
        return false;
    }
    let year = value[0..4].parse::<u32>().unwrap_or_default();
    let month = value[5..7].parse::<u32>().unwrap_or_default();
    let day = value[8..10].parse::<u32>().unwrap_or_default();
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => return false,
    };
    (1..=max_day).contains(&day)
}

fn is_standard_uk_quarter(start: &str, end: &str) -> bool {
    if !is_calendar_date(start) || !is_calendar_date(end) {
        return false;
    }
    let year = start[0..4].parse::<u32>().unwrap_or_default();
    let expected = match &start[5..] {
        "04-06" => format!("{year:04}-07-05"),
        "07-06" => format!("{year:04}-10-05"),
        "10-06" => format!("{:04}-01-05", year + 1),
        "01-06" => format!("{year:04}-04-05"),
        _ => return false,
    };
    end == expected
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

async fn write_audit_transaction(
    transaction: &mut Transaction<'_, Sqlite>,
    workspace: &str,
    action: &str,
    detail: &[u8],
) -> Result<(), ApiError> {
    let previous =
        sqlx::query("SELECT hash FROM audit_log WHERE workspace_id = ? ORDER BY id DESC LIMIT 1")
            .bind(workspace)
            .fetch_optional(&mut **transaction)
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
        .execute(&mut **transaction)
        .await
        .map_err(internal)?;
    Ok(())
}

async fn persist_state(state: &AppState) -> Result<(), ApiError> {
    persist_database_snapshot(&state.database_path, &state.snapshot_path)
        .await
        .map_err(internal)
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
        // Azure Files can be mounted without POSIX chmod support. The share
        // is private to the Container App; do not turn a supported security
        // tightening into an availability outage when the mount rejects it.
        if let Err(error) = fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).await
        {
            warn!(%error, path = %path.display(), "key_permission_mode_not_supported");
        }
    }
    Ok((key, true))
}

async fn restore_database_snapshot(
    snapshot: &FsPath,
    database: &FsPath,
) -> Result<(), std::io::Error> {
    if fs::metadata(snapshot).await.is_ok() && fs::metadata(database).await.is_err() {
        // `std::fs::copy` also attempts to reproduce permissions from the
        // source. Azure Files accepts the bytes but can reject that metadata
        // operation with EPERM, leaving a zero-byte destination behind.
        let snapshot_bytes = fs::read(snapshot).await?;
        fs::write(database, snapshot_bytes).await?;
    }
    Ok(())
}

async fn persist_database_snapshot(
    database: &FsPath,
    snapshot: &FsPath,
) -> Result<(), std::io::Error> {
    if fs::metadata(database).await.is_err() {
        return Ok(());
    }

    // Copy to a unique sibling first. A crash or failed Azure Files write
    // therefore leaves the last complete snapshot intact instead of
    // truncating the only durable copy. This runs while the process-wide
    // mutation lock is held; sync_all and the atomic rename both complete
    // before the route returns success.
    let mut source = fs::File::open(database).await?;
    let temporary = snapshot.with_extension(format!("sqlite3.tmp-{}", Uuid::new_v4().simple()));
    let copy_result = async {
        let mut destination = fs::File::create(&temporary).await?;
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let bytes = source.read(&mut buffer).await?;
            if bytes == 0 {
                break;
            }
            destination.write_all(&buffer[..bytes]).await?;
        }
        destination.sync_all().await
    }
    .await;
    if let Err(error) = copy_result {
        let _ = fs::remove_file(&temporary).await;
        return Err(error);
    }
    if let Err(error) = fs::rename(&temporary, snapshot).await {
        let _ = fs::remove_file(&temporary).await;
        return Err(error);
    }
    Ok(())
}

async fn migrate(db: &SqlitePool) -> Result<(), sqlx::Error> {
    // M1 browser workspaces remain readable so an existing customer can choose
    // to move a quarter. M2 account data uses the tables below; its authority
    // is a signed OIDC session plus membership, never a browser UUID.
    sqlx::query("CREATE TABLE IF NOT EXISTS workspaces(id TEXT PRIMARY KEY, payload BLOB NOT NULL, updated_at INTEGER NOT NULL)").execute(db).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS shares(token TEXT PRIMARY KEY, workspace_id TEXT NOT NULL, payload BLOB NOT NULL, expires_at INTEGER NOT NULL)").execute(db).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS audit_log(id INTEGER PRIMARY KEY AUTOINCREMENT, workspace_id TEXT NOT NULL, action TEXT NOT NULL, created_at INTEGER NOT NULL, hash TEXT NOT NULL)").execute(db).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS hmrc_consents(workspace_id TEXT PRIMARY KEY, payload BLOB NOT NULL, expires_at INTEGER NOT NULL, created_at INTEGER NOT NULL)").execute(db).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS hmrc_consent_states(state TEXT PRIMARY KEY, workspace_id TEXT NOT NULL, expires_at INTEGER NOT NULL)").execute(db).await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS page_views(day INTEGER PRIMARY KEY, count INTEGER NOT NULL)",
    )
    .execute(db)
    .await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS users(subject TEXT PRIMARY KEY, display_name TEXT NOT NULL, email TEXT, created_at INTEGER NOT NULL)")
        .execute(db)
        .await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS sessions(token_hash TEXT PRIMARY KEY, subject TEXT NOT NULL, expires_at INTEGER NOT NULL, created_at INTEGER NOT NULL)")
        .execute(db)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS sessions_subject ON sessions(subject)")
        .execute(db)
        .await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS auth_states(state TEXT PRIMARY KEY, nonce TEXT NOT NULL, code_verifier TEXT NOT NULL, expires_at INTEGER NOT NULL)")
        .execute(db)
        .await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS businesses(id TEXT PRIMARY KEY, owner_subject TEXT NOT NULL, name TEXT NOT NULL, created_at INTEGER NOT NULL)")
        .execute(db)
        .await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS memberships(business_id TEXT NOT NULL, subject TEXT NOT NULL, role TEXT NOT NULL, created_at INTEGER NOT NULL, PRIMARY KEY(business_id, subject))")
        .execute(db)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS memberships_subject ON memberships(subject)")
        .execute(db)
        .await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS account_quarters(business_id TEXT NOT NULL, quarter_start TEXT NOT NULL, payload BLOB NOT NULL, updated_at INTEGER NOT NULL, PRIMARY KEY(business_id, quarter_start))")
        .execute(db)
        .await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS account_migrations(business_id TEXT NOT NULL, quarter_start TEXT NOT NULL, migration_id TEXT NOT NULL, created_at INTEGER NOT NULL, PRIMARY KEY(business_id, quarter_start, migration_id))")
        .execute(db)
        .await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS account_shares(token TEXT PRIMARY KEY, business_id TEXT NOT NULL, quarter_start TEXT NOT NULL, payload BLOB NOT NULL, expires_at INTEGER NOT NULL)")
        .execute(db)
        .await?;
    Ok(())
}

async fn migrate_with_retry(db: &SqlitePool) -> Result<(), sqlx::Error> {
    const MAX_ATTEMPTS: u8 = 6;
    for attempt in 1..=MAX_ATTEMPTS {
        match migrate(db).await {
            Ok(()) => return Ok(()),
            Err(error)
                if is_database_locked_message(&error.to_string()) && attempt < MAX_ATTEMPTS =>
            {
                warn!(attempt, "database_locked_during_startup_migration");
                tokio::time::sleep(Duration::from_millis(500 * u64::from(attempt))).await;
            }
            Err(error) => return Err(error),
        }
    }
    unreachable!("migration loop returns on success or final error")
}

fn is_database_locked_message(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("database is locked") || message.contains("database is busy")
}

async fn cleanup_expired_shares(db: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM shares WHERE expires_at < ?")
        .bind(unix_now() as i64)
        .execute(db)
        .await?;
    sqlx::query("DELETE FROM hmrc_consents WHERE expires_at < ?")
        .bind(unix_now() as i64)
        .execute(db)
        .await?;
    sqlx::query("DELETE FROM hmrc_consent_states WHERE expires_at < ?")
        .bind(unix_now() as i64)
        .execute(db)
        .await?;
    sqlx::query("DELETE FROM sessions WHERE expires_at < ?")
        .bind(unix_now() as i64)
        .execute(db)
        .await?;
    sqlx::query("DELETE FROM auth_states WHERE expires_at < ?")
        .bind(unix_now() as i64)
        .execute(db)
        .await?;
    sqlx::query("DELETE FROM account_shares WHERE expires_at < ?")
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

fn default_log_filter() -> tracing_subscriber::EnvFilter {
    log_filter(env::var("RUST_LOG").ok())
}

fn log_filter(value: Option<String>) -> tracing_subscriber::EnvFilter {
    value
        .and_then(|filter| tracing_subscriber::EnvFilter::try_new(filter).ok())
        .unwrap_or_else(|| tracing_subscriber::EnvFilter::new("info"))
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
        assert!(validate_document(&json!({
            "quarterStart": "2026-04-06", "quarterEnd": "2026-07-05", "transactions":[]
        }))
        .is_ok());
        assert!(validate_document(&json!({"items":[]})).is_err());
        assert!(validate_document(&json!({
            "quarterStart": "2026-02-30", "quarterEnd": "2026-07-05", "transactions":[]
        }))
        .is_err());
        assert!(validate_document(&json!({
            "quarterStart": "2026-04-06", "quarterEnd": "2026-07-06", "transactions":[]
        }))
        .is_err());
    }

    #[test]
    fn rejects_each_malformed_transaction_field() {
        let valid = json!({
            "id": "record-1", "date": "2026-04-09", "description": "Maths lesson",
            "amountPence": 4500, "kind": "income", "category": "Sales"
        });
        for invalid in [
            json!({"date": "2026-04-09", "description": "Maths lesson", "amountPence": 4500, "kind": "income", "category": "Sales"}),
            json!({"id": "record-1", "date": "2026-02-30", "description": "Maths lesson", "amountPence": 4500, "kind": "income", "category": "Sales"}),
            json!({"id": "record-1", "date": "2026-04-09", "description": "", "amountPence": 4500, "kind": "income", "category": "Sales"}),
            json!({"id": "record-1", "date": "2026-04-09", "description": "Maths lesson", "amountPence": 0, "kind": "income", "category": "Sales"}),
            json!({"id": "record-1", "date": "2026-04-09", "description": "Maths lesson", "amountPence": 4500, "kind": "transfer", "category": "Sales"}),
            json!({"id": "record-1", "date": "2026-04-09", "description": "Maths lesson", "amountPence": 4500, "kind": "income", "category": "Unknown"}),
        ] {
            assert!(validate_document(&json!({
                "quarterStart": "2026-04-06", "quarterEnd": "2026-07-05", "transactions": [invalid]
            }))
            .is_err());
        }
        assert!(validate_document(&json!({
            "quarterStart": "2026-04-06", "quarterEnd": "2026-07-05", "transactions": [valid.clone(), valid]
        }))
        .is_err());
    }

    #[test]
    fn rejects_transactions_outside_the_selected_quarter() {
        for date in ["2026-04-05", "2026-07-06"] {
            let document = json!({
                "quarterStart": "2026-04-06", "quarterEnd": "2026-07-05",
                "transactions": [{
                    "id": "record-1", "date": date, "description": "Maths lesson",
                    "amountPence": 4500, "kind": "income", "category": "Sales"
                }]
            });
            assert!(validate_document(&document).is_err(), "accepted {date}");
        }
    }

    async fn account_test_state() -> (AppState, PathBuf) {
        let root =
            std::env::temp_dir().join(format!("quarterly-ready-accounts-{}", Uuid::new_v4()));
        let data = root.join("data");
        let local = root.join("local");
        fs::create_dir_all(&data).await.unwrap();
        fs::create_dir_all(&local).await.unwrap();
        let database_path = local.join("quarterly-ready.sqlite3");
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&format!("sqlite://{}?mode=rwc", database_path.display()))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        (
            AppState {
                db,
                key: [11u8; 32],
                database_path,
                snapshot_path: data.join("quarterly-ready.snapshot.sqlite3"),
                write_lock: Arc::new(Mutex::new(())),
                client: reqwest::Client::new(),
                billing_base_url: "https://api.sociobot.in/api/v1".into(),
                auth: None,
                hmrc_integration: None,
                safe_qa_fixtures: false,
            },
            root,
        )
    }

    fn account_request(token: &str) -> Request<Body> {
        Request::builder()
            .header(header::COOKIE, format!("{SESSION_COOKIE}={token}"))
            .body(Body::empty())
            .unwrap()
    }

    fn account_document(description: &str) -> Value {
        json!({
            "schemaVersion": 1,
            "businessName": "Maya Patel Tutoring",
            "quarterLabel": "6 April to 5 July 2026",
            "quarterStart": "2026-04-06",
            "quarterEnd": "2026-07-05",
            "figuresReviewed": false,
            "packDownloaded": false,
            "markedReady": false,
            "updatedAt": "2026-04-09T12:00:00.000Z",
            "transactions": [{
                "id": "account-record-1",
                "date": "2026-04-09",
                "description": description,
                "amountPence": 4500,
                "kind": "income",
                "category": "Sales"
            }]
        })
    }

    async fn seed_account(
        state: &AppState,
        subject: &str,
        token: &str,
        business_id: &str,
        business_name: &str,
        document: &Value,
    ) {
        sqlx::query(
            "INSERT INTO users(subject, display_name, email, created_at) VALUES(?, ?, ?, ?)",
        )
        .bind(subject)
        .bind(subject)
        .bind(format!("{subject}@example.test"))
        .bind(1_i64)
        .execute(&state.db)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO sessions(token_hash, subject, expires_at, created_at) VALUES(?, ?, ?, ?)",
        )
        .bind(token_hash(token))
        .bind(subject)
        .bind((unix_now() + 3600) as i64)
        .bind(1_i64)
        .execute(&state.db)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO businesses(id, owner_subject, name, created_at) VALUES(?, ?, ?, ?)",
        )
        .bind(business_id)
        .bind(subject)
        .bind(business_name)
        .bind(1_i64)
        .execute(&state.db)
        .await
        .unwrap();
        sqlx::query("INSERT INTO memberships(business_id, subject, role, created_at) VALUES(?, ?, 'owner', ?)")
            .bind(business_id)
            .bind(subject)
            .bind(1_i64)
            .execute(&state.db)
            .await
            .unwrap();
        sqlx::query("INSERT INTO account_quarters(business_id, quarter_start, payload, updated_at) VALUES(?, ?, ?, ?)")
            .bind(business_id)
            .bind("2026-04-06")
            .bind(encrypt_json(&state.key, document).unwrap())
            .bind(1_i64)
            .execute(&state.db)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn claim_tenant_isolation() {
        let (state, root) = account_test_state().await;
        let first_business = Uuid::new_v4().to_string();
        let second_business = Uuid::new_v4().to_string();
        let first_document = account_document("First business lesson");
        let second_document = account_document("Second business lesson");
        seed_account(
            &state,
            "subject-one",
            "first-session",
            &first_business,
            "First business",
            &first_document,
        )
        .await;
        seed_account(
            &state,
            "subject-two",
            "second-session",
            &second_business,
            "Second business",
            &second_document,
        )
        .await;

        let owner_result = get_account_quarter(
            State(state.clone()),
            Path((first_business.clone(), "2026-04-06".into())),
            account_request("first-session"),
        )
        .await
        .unwrap()
        .0;
        assert_eq!(owner_result.document.unwrap(), first_document);

        let other_user = get_account_quarter(
            State(state.clone()),
            Path((first_business, "2026-04-06".into())),
            account_request("second-session"),
        )
        .await
        .unwrap_err();
        assert_eq!(other_user.0, StatusCode::NOT_FOUND);
        let second_result = get_account_quarter(
            State(state.clone()),
            Path((second_business, "2026-04-06".into())),
            account_request("second-session"),
        )
        .await
        .unwrap()
        .0;
        assert_eq!(second_result.document.unwrap(), second_document);
        state.db.close().await;
        fs::remove_dir_all(root).await.unwrap();
    }

    #[tokio::test]
    async fn claim_browser_quarter_migration_is_explicit_and_idempotent() {
        let (state, root) = account_test_state().await;
        let business = Uuid::new_v4().to_string();
        let original = account_document("Browser quarter lesson");
        seed_account(
            &state,
            "subject-one",
            "first-session",
            &business,
            "First business",
            &account_document("Older server lesson"),
        )
        .await;
        let migration_id = Uuid::new_v4().to_string();
        let request = Request::builder()
            .header(header::COOKIE, format!("{SESSION_COOKIE}=first-session"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::to_vec(&json!({ "document": original, "migration_id": migration_id }))
                    .unwrap(),
            ))
            .unwrap();
        let moved = put_account_quarter(
            State(state.clone()),
            Path((business.clone(), "2026-04-06".into())),
            request,
        )
        .await
        .unwrap()
        .0;
        assert_eq!(moved.document, Some(original.clone()));

        let altered = account_document("Retry must not overwrite the first moved quarter");
        let retry = Request::builder()
            .header(header::COOKIE, format!("{SESSION_COOKIE}=first-session"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::to_vec(&json!({ "document": altered, "migration_id": migration_id }))
                    .unwrap(),
            ))
            .unwrap();
        let repeated = put_account_quarter(
            State(state.clone()),
            Path((business.clone(), "2026-04-06".into())),
            retry,
        )
        .await
        .unwrap()
        .0;
        assert_eq!(repeated.document, Some(original.clone()));
        let migration_count: i64 =
            sqlx::query("SELECT COUNT(*) AS count FROM account_migrations WHERE business_id = ?")
                .bind(&business)
                .fetch_one(&state.db)
                .await
                .unwrap()
                .get("count");
        assert_eq!(migration_count, 1);
        state.db.close().await;
        fs::remove_dir_all(root).await.unwrap();
    }

    #[tokio::test]
    async fn claim_account_export_and_delete() {
        let (state, root) = account_test_state().await;
        let first_business = Uuid::new_v4().to_string();
        let second_business = Uuid::new_v4().to_string();
        let first_document = account_document("Exported lesson");
        let second_document = account_document("Other account lesson");
        seed_account(
            &state,
            "subject-one",
            "first-session",
            &first_business,
            "First business",
            &first_document,
        )
        .await;
        seed_account(
            &state,
            "subject-two",
            "second-session",
            &second_business,
            "Second business",
            &second_document,
        )
        .await;

        let export = export_account(State(state.clone()), account_request("first-session"))
            .await
            .unwrap()
            .0;
        assert_eq!(export["format"], "quarterly-ready-account-export-v1");
        assert_eq!(export["businesses"].as_array().unwrap().len(), 1);
        assert_eq!(
            export["businesses"][0]["quarters"][0]["document"],
            first_document
        );

        assert_eq!(
            delete_account(State(state.clone()), account_request("first-session"))
                .await
                .unwrap(),
            StatusCode::NO_CONTENT
        );
        let first_user_count: i64 =
            sqlx::query("SELECT COUNT(*) AS count FROM users WHERE subject = 'subject-one'")
                .fetch_one(&state.db)
                .await
                .unwrap()
                .get("count");
        let first_quarter_count: i64 =
            sqlx::query("SELECT COUNT(*) AS count FROM account_quarters WHERE business_id = ?")
                .bind(&first_business)
                .fetch_one(&state.db)
                .await
                .unwrap()
                .get("count");
        let second_quarter = stored_account_quarter(&state, &second_business, "2026-04-06")
            .await
            .unwrap();
        assert_eq!(first_user_count, 0);
        assert_eq!(first_quarter_count, 0);
        assert_eq!(second_quarter, Some(second_document));
        state.db.close().await;
        fs::remove_dir_all(root).await.unwrap();
    }

    #[test]
    fn startup_migration_retries_transient_sqlite_locks() {
        assert!(is_database_locked_message("database is locked"));
        assert!(is_database_locked_message("SQLITE_BUSY: database is busy"));
        assert!(!is_database_locked_message("no such table"));
    }

    #[tokio::test]
    async fn mounted_snapshot_is_imported_to_local_sqlite_without_copying_permissions() {
        let root =
            std::env::temp_dir().join(format!("quarterly-ready-snapshot-{}", Uuid::new_v4()));
        let durable = root.join("durable");
        fs::create_dir_all(&durable).await.unwrap();
        let database = durable.join("quarterly-ready.sqlite3");
        let snapshot = durable.join("quarterly-ready.snapshot.sqlite3");
        fs::write(&snapshot, b"legacy encrypted workspace snapshot")
            .await
            .unwrap();
        restore_database_snapshot(&snapshot, &database)
            .await
            .unwrap();
        assert_eq!(
            fs::read(&database).await.unwrap(),
            b"legacy encrypted workspace snapshot"
        );
        fs::remove_dir_all(root).await.unwrap();
    }

    #[tokio::test]
    async fn durable_database_restores_key_records_links_audit_and_page_count_after_restart() {
        let root = std::env::temp_dir().join(format!("quarterly-ready-state-{}", Uuid::new_v4()));
        let durable = root.join("durable");
        let local = root.join("local");
        fs::create_dir_all(&durable).await.unwrap();
        fs::create_dir_all(&local).await.unwrap();
        let database_path = local.join("quarterly-ready.sqlite3");
        let snapshot_path = durable.join("quarterly-ready.snapshot.sqlite3");
        let key_path = durable.join("quarterly-ready.key");
        let (key, generated) = load_or_create_key(&key_path).await.unwrap();
        assert!(generated);
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&format!("sqlite://{}?mode=rwc", database_path.display()))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let document = json!({
            "quarterStart": "2026-04-06", "quarterEnd": "2026-07-05",
            "transactions": [{"id":"durable-1","date":"2026-04-09","description":"Durable lesson","amountPence":4500,"kind":"income","category":"Sales"}]
        });
        let encrypted = encrypt_json(&key, &document).unwrap();
        sqlx::query("INSERT INTO workspaces(id, payload, updated_at) VALUES(?, ?, ?)")
            .bind("durable-workspace")
            .bind(&encrypted)
            .bind(1_i64)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO shares(token, workspace_id, payload, expires_at) VALUES(?, ?, ?, ?)",
        )
        .bind("durable-share")
        .bind("durable-workspace")
        .bind(&encrypted)
        .bind(i64::MAX)
        .execute(&db)
        .await
        .unwrap();
        let state = AppState {
            db,
            key,
            database_path: database_path.clone(),
            snapshot_path: snapshot_path.clone(),
            write_lock: Arc::new(Mutex::new(())),
            client: reqwest::Client::new(),
            billing_base_url: "https://api.sociobot.in/api/v1".into(),
            auth: None,
            hmrc_integration: None,
            safe_qa_fixtures: false,
        };
        write_audit(&state, "durable-workspace", "records_saved", b"durable")
            .await
            .unwrap();
        sqlx::query("INSERT INTO page_views(day, count) VALUES(?, ?)")
            .bind(1_i64)
            .bind(3_i64)
            .execute(&state.db)
            .await
            .unwrap();
        persist_state(&state).await.unwrap();
        state.db.close().await;
        fs::remove_file(&database_path).await.unwrap();
        restore_database_snapshot(&snapshot_path, &database_path)
            .await
            .unwrap();

        let (restored_key, regenerated) = load_or_create_key(&key_path).await.unwrap();
        assert!(!regenerated);
        assert_eq!(restored_key, key);
        let restored = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&format!("sqlite://{}?mode=rw", database_path.display()))
            .await
            .unwrap();
        for table in ["workspaces", "shares"] {
            let query = format!("SELECT payload FROM {table} LIMIT 1");
            let payload: Vec<u8> = sqlx::query(&query)
                .fetch_one(&restored)
                .await
                .unwrap()
                .get("payload");
            assert_eq!(decrypt_json(&restored_key, &payload).unwrap(), document);
        }
        let audit_count: i64 = sqlx::query("SELECT COUNT(*) AS count FROM audit_log")
            .fetch_one(&restored)
            .await
            .unwrap()
            .get("count");
        let page_count: i64 = sqlx::query("SELECT count FROM page_views WHERE day = 1")
            .fetch_one(&restored)
            .await
            .unwrap()
            .get("count");
        assert_eq!(audit_count, 1);
        assert_eq!(page_count, 3);
        restored.close().await;
        fs::remove_dir_all(root).await.unwrap();
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
            database_path: PathBuf::from("/tmp/quarterly-ready-test.sqlite3"),
            snapshot_path: PathBuf::from("/tmp/quarterly-ready-test.snapshot.sqlite3"),
            write_lock: Arc::new(Mutex::new(())),
            client: reqwest::Client::new(),
            billing_base_url: "https://api.sociobot.in/api/v1".into(),
            auth: None,
            hmrc_integration: None,
            safe_qa_fixtures: false,
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

    #[test]
    fn default_log_filter_keeps_startup_information_visible() {
        assert_eq!(log_filter(None).to_string(), "info");
    }

    #[test]
    fn account_sign_in_requires_valid_public_oidc_configuration_and_uses_pkce() {
        assert!(auth_config_from_values(
            Some("https://identity.example.test/tenant".into()),
            Some("quarterly-ready-web".into()),
            None,
        )
        .is_some());
        assert!(auth_config_from_values(
            Some("http://identity.example.test".into()),
            Some("quarterly-ready-web".into()),
            None,
        )
        .is_none());
        assert!(auth_config_from_values(
            Some("https://identity.example.test".into()),
            Some("".into()),
            None,
        )
        .is_none());
        assert_eq!(
            pkce_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
        assert!(tokens_equal("browser-bound-state", "browser-bound-state"));
        assert!(!tokens_equal("browser-bound-state", "other-browser-state"));
        assert!(!tokens_equal("short", "longer"));
    }

    #[test]
    fn claim_accountant_link_expiry_is_thirty_days() {
        assert_eq!(SHARE_EXPIRY_SECONDS, 30 * 24 * 60 * 60);
    }

    #[test]
    fn hmrc_payload_requires_explicit_human_review() {
        let error = hmrc_compatible_payload(
            &json!({
                "quarterStart": "2026-04-06", "quarterEnd": "2026-07-05", "transactions": []
            }),
            false,
        )
        .unwrap_err();
        assert_eq!(error.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            error.1,
            "Confirm that you reviewed the totals before submitting to HMRC."
        );
    }

    #[test]
    fn sandbox_configuration_requires_the_official_non_filing_hmrc_endpoint() {
        let token = Some("key-vault-attestation".to_owned());
        let mode = Some("hmrc_sandbox_no_filing".to_owned());
        assert!(approved_integration_from_values(
            Some("https://not-hmrc.example/sandbox".to_owned()),
            token.clone(),
            mode.clone(),
            None,
        )
        .is_none());
        let integration = approved_integration_from_values(
            Some("https://test-api.service.hmrc.gov.uk/hello/world".to_owned()),
            token,
            mode,
            None,
        )
        .expect("official HMRC test endpoint should configure sandbox mode");
        assert_eq!(integration.mode, IntegrationMode::HmrcSandboxNoFiling);
    }

    #[test]
    fn approved_provider_configuration_requires_taxpayer_consent_and_an_approval_reference() {
        let no_consent = approved_integration_from_values(
            Some("https://provider.example/periodic-updates".to_owned()),
            Some("provider-service-token".to_owned()),
            Some("approved_provider".to_owned()),
            None,
        );
        assert!(no_consent.is_none());
        let consent = taxpayer_consent_from_values(
            Some("https://provider.example/authorize".to_owned()),
            Some("https://provider.example/token".to_owned()),
            Some("registered-client".to_owned()),
            Some("registered-secret".to_owned()),
            Some("https://mtd-quarterly-ready.sociobot.in/api/hmrc/consent/callback".to_owned()),
            Some("Approved MTD provider".to_owned()),
            Some("HMRC-approved-software-reference".to_owned()),
        );
        let integration = approved_integration_from_values(
            Some("https://provider.example/periodic-updates".to_owned()),
            Some("provider-service-token".to_owned()),
            Some("approved_provider".to_owned()),
            consent,
        )
        .expect("approved provider requires an explicit OAuth consent configuration");
        assert_eq!(integration.mode, IntegrationMode::ApprovedProvider);
        assert_eq!(
            integration
                .taxpayer_consent
                .as_ref()
                .unwrap()
                .provider_approval_reference,
            "HMRC-approved-software-reference"
        );
    }

    #[tokio::test]
    async fn taxpayer_consent_uses_oauth_state_encrypts_the_token_and_marks_the_workspace_ready() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::sync::oneshot;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (request_tx, request_rx) = oneshot::channel();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut bytes = vec![0; 4096];
            let read = stream.read(&mut bytes).await.unwrap();
            request_tx
                .send(String::from_utf8_lossy(&bytes[..read]).to_string())
                .unwrap();
            let body = r#"{"access_token":"taxpayer-token","refresh_token":"refresh-token","expires_in":7200}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(), body
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let state = AppState {
            db,
            key: [5u8; 32],
            database_path: PathBuf::from("/tmp/quarterly-ready-test.sqlite3"),
            snapshot_path: PathBuf::from("/tmp/quarterly-ready-test.snapshot.sqlite3"),
            write_lock: Arc::new(Mutex::new(())),
            client: reqwest::Client::new(),
            billing_base_url: "https://api.sociobot.in/api/v1".into(),
            auth: None,
            hmrc_integration: Some(ApprovedIntegration {
                url: "https://provider.example/periodic-updates".into(),
                token: "provider-service-token".into(),
                mode: IntegrationMode::ApprovedProvider,
                taxpayer_consent: Some(TaxpayerConsent {
                    authorize_url: "https://provider.example/authorize".into(),
                    token_url: format!("http://{address}/token"),
                    client_id: "registered-client".into(),
                    client_secret: "registered-secret".into(),
                    redirect_uri: "https://quarterly-ready.example/api/hmrc/consent/callback"
                        .into(),
                    provider_name: "Approved MTD provider".into(),
                    provider_approval_reference: "approved-reference".into(),
                }),
            }),
            safe_qa_fixtures: false,
        };
        let workspace_id = "65aa583d-84cf-43f1-8438-354ddbfd6358";
        let start_request = Request::builder()
            .header("x-workspace-id", workspace_id)
            .body(Body::empty())
            .unwrap();
        let started = start_hmrc_consent(State(state.clone()), start_request)
            .await
            .unwrap()
            .0;
        let authorization_url = Url::parse(&started.authorization_url).unwrap();
        assert_eq!(
            authorization_url.origin().ascii_serialization(),
            "https://provider.example"
        );
        let parameters: HashMap<String, String> =
            authorization_url.query_pairs().into_owned().collect();
        assert_eq!(
            parameters.get("response_type").map(String::as_str),
            Some("code")
        );
        assert_eq!(
            parameters.get("client_id").map(String::as_str),
            Some("registered-client")
        );
        assert_eq!(
            parameters.get("scope").map(String::as_str),
            Some("write:self-assessment")
        );
        let oauth_state = parameters.get("state").cloned().unwrap();
        let callback = hmrc_consent_callback(
            State(state.clone()),
            Query(ConsentCallback {
                state: oauth_state,
                code: Some("authorisation-code".into()),
                error: None,
            }),
        )
        .await
        .unwrap();
        assert_eq!(
            callback
                .into_response()
                .headers()
                .get(header::LOCATION)
                .unwrap(),
            "/records?hmrc-consent=connected"
        );
        let status_request = Request::builder()
            .header("x-workspace-id", workspace_id)
            .body(Body::empty())
            .unwrap();
        let status = hmrc_consent_status(State(state.clone()), status_request)
            .await
            .unwrap()
            .0;
        assert!(status.consented);
        assert!(status.expires_at.unwrap() > unix_now());
        let stored: Vec<u8> =
            sqlx::query("SELECT payload FROM hmrc_consents WHERE workspace_id = ?")
                .bind(workspace_id)
                .fetch_one(&state.db)
                .await
                .unwrap()
                .get("payload");
        assert!(!String::from_utf8_lossy(&stored).contains("taxpayer-token"));
        let request = request_rx.await.unwrap();
        assert!(request.starts_with("POST /token"));
        assert!(request
            .contains("authorization: Basic cmVnaXN0ZXJlZC1jbGllbnQ6cmVnaXN0ZXJlZC1zZWNyZXQ="));
        assert!(request.contains("grant_type=authorization_code"));
        assert!(request.contains("code=authorisation-code"));
    }

    #[tokio::test]
    async fn claim_hmrc_sandbox_is_non_filing_and_sends_no_records_or_secret() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::sync::oneshot;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (request_tx, request_rx) = oneshot::channel();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut bytes = vec![0; 4096];
            let read = stream.read(&mut bytes).await.unwrap();
            request_tx
                .send(String::from_utf8_lossy(&bytes[..read]).to_string())
                .unwrap();
            let body = r#"{"message":"Hello World"}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(), body
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });
        let payload = json!({
            "format": "quarterly-ready-mtd-itsa-periodic-update-v1",
            "periodIncome": { "turnover": 260.0 },
            "reviewedByUser": true
        });
        let result = send_to_approved_integration(
            &reqwest::Client::new(),
            &ApprovedIntegration {
                url: format!("http://{address}/hello/world"),
                token: "sandbox-attestation-must-stay-server-side".into(),
                mode: IntegrationMode::HmrcSandboxNoFiling,
                taxpayer_consent: None,
            },
            &payload,
            None,
        )
        .await
        .unwrap();
        assert_eq!(result.status, "sandbox_accepted_no_filing");
        assert!(!result.files_with_hmrc);
        assert!(result.submission_id.starts_with("hmrc-sandbox-no-filing-"));
        let request = request_rx.await.unwrap();
        assert!(request.starts_with("GET /hello/world"));
        assert!(request.contains("application/vnd.hmrc.1.0+json"));
        assert!(!request.contains("periodIncome"));
        assert!(!request.contains("sandbox-attestation-must-stay-server-side"));
        assert!(!request.to_ascii_lowercase().contains("authorization:"));
    }

    #[tokio::test]
    async fn subscription_verification_accepts_the_annual_entitlement() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            for valid in [false, true] {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut bytes = vec![0; 2048];
                let read = stream.read(&mut bytes).await.unwrap();
                let request = String::from_utf8_lossy(&bytes[..read]);
                let body = format!(r#"{{"valid":{valid}}}"#);
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    body.len(), body
                );
                stream.write_all(response.as_bytes()).await.unwrap();
                if valid {
                    assert!(request.starts_with(
                        "GET /products/mtd-quarterly-ready-annual/verify?license=annual-token"
                    ));
                } else {
                    assert!(request.starts_with(
                        "GET /products/mtd-quarterly-ready/verify?license=annual-token"
                    ));
                }
            }
        });
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let state = AppState {
            db,
            key: [3u8; 32],
            database_path: PathBuf::from("/tmp/quarterly-ready-test.sqlite3"),
            snapshot_path: PathBuf::from("/tmp/quarterly-ready-test.snapshot.sqlite3"),
            write_lock: Arc::new(Mutex::new(())),
            client: reqwest::Client::new(),
            billing_base_url: format!("http://{address}"),
            auth: None,
            hmrc_integration: None,
            safe_qa_fixtures: false,
        };
        verify_licence_token(&state, "annual-token").await.unwrap();
        server.await.unwrap();
    }

    #[tokio::test]
    async fn claim_hmrc_submission_uses_an_approved_integration_after_human_review() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::sync::mpsc;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (request_tx, mut request_rx) = mpsc::channel(2);
        tokio::spawn(async move {
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut bytes = vec![0; 8192];
                let read = stream.read(&mut bytes).await.unwrap();
                let request = String::from_utf8_lossy(&bytes[..read]).to_string();
                request_tx.send(request.clone()).await.unwrap();
                let body = if request.starts_with("GET /products/") {
                    r#"{"valid":true}"#
                } else {
                    r#"{"submission_id":"mtd-test-123"}"#
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    body.len(), body
                );
                stream.write_all(response.as_bytes()).await.unwrap();
            }
        });
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let state = AppState {
            db,
            key: [4u8; 32],
            database_path: PathBuf::from("/tmp/quarterly-ready-test.sqlite3"),
            snapshot_path: PathBuf::from("/tmp/quarterly-ready-test.snapshot.sqlite3"),
            write_lock: Arc::new(Mutex::new(())),
            client: reqwest::Client::new(),
            billing_base_url: format!("http://{address}"),
            auth: None,
            hmrc_integration: Some(ApprovedIntegration {
                url: format!("http://{address}/submit"),
                token: "bridge-secret".into(),
                mode: IntegrationMode::ApprovedProvider,
                taxpayer_consent: Some(TaxpayerConsent {
                    authorize_url: "https://approved-provider.test/authorize".into(),
                    token_url: "https://approved-provider.test/token".into(),
                    client_id: "provider-client-id".into(),
                    client_secret: "provider-client-secret".into(),
                    redirect_uri: "https://quarterly-ready.test/api/hmrc/consent/callback".into(),
                    provider_name: "Approved provider test fixture".into(),
                    provider_approval_reference: "test-approved-reference".into(),
                }),
            }),
            safe_qa_fixtures: false,
        };
        let consent = encrypt_json(
            &state.key,
            &json!({ "access_token": "taxpayer-consent-token" }),
        )
        .unwrap();
        sqlx::query("INSERT INTO hmrc_consents(workspace_id, payload, expires_at, created_at) VALUES(?, ?, ?, ?)")
            .bind("15aa583d-84cf-43f1-8438-354ddbfd6358")
            .bind(consent)
            .bind((unix_now() + 3600) as i64)
            .bind(unix_now() as i64)
            .execute(&state.db)
            .await
            .unwrap();
        let document = json!({
            "quarterStart": "2026-04-06", "quarterEnd": "2026-07-05",
            "figuresReviewed": true, "markedReady": true,
            "transactions": [{ "id": "income-1", "date": "2026-04-09", "description": "Lesson income", "amountPence": 26000, "kind": "income", "category": "Sales" }]
        });
        let request = Request::builder()
            .header("x-workspace-id", "15aa583d-84cf-43f1-8438-354ddbfd6358")
            .header("x-sociobot-license", "active-subscription")
            .body(Body::from(
                json!({ "document": document, "review_confirmed": true }).to_string(),
            ))
            .unwrap();
        let result = submit_to_hmrc(State(state), request).await.unwrap().0;
        assert_eq!(result.submission_id, "mtd-test-123");
        assert_eq!(result.status, "accepted");
        assert!(result.files_with_hmrc);
        assert!(request_rx
            .recv()
            .await
            .unwrap()
            .starts_with("GET /products/mtd-quarterly-ready/verify?license=active-subscription"));
        let integration_request = request_rx.recv().await.unwrap();
        assert!(integration_request.starts_with("POST /submit"));
        assert!(integration_request.contains("quarterly-ready-mtd-itsa-periodic-update-v1"));
        assert!(integration_request.contains("authorization: Bearer taxpayer-consent-token"));
        assert!(integration_request.contains("x-quarterly-ready-provider-token: bridge-secret"));
    }
}
