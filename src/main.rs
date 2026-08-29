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
    routing::{get, get_service, post},
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
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    signal,
    sync::Mutex,
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
const SAFE_FIXTURE_TOKEN: &str = "quarterly-ready-safe-no-charge-fixture-v1";
const SAFE_FIXTURE_BUSINESS: &str = "Quarterly Ready safe QA fixture";
const SQLITE_BUSY_TIMEOUT_MS: u64 = 30_000;
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
    persistence: Arc<Mutex<()>>,
    limits: Arc<Mutex<HashMap<String, VecDeque<Instant>>>>,
    client: reqwest::Client,
    billing_base_url: String,
    hmrc_integration: Option<ApprovedIntegration>,
    safe_qa_fixtures: bool,
}

#[derive(Clone)]
struct ApprovedIntegration {
    url: String,
    token: String,
}

#[derive(Serialize)]
struct Health<'a> {
    status: &'a str,
    build_sha: &'a str,
    safe_qa_fixtures: bool,
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
struct LicenceVerdict {
    valid: bool,
}

#[derive(Serialize)]
struct SubmissionResult {
    submission_id: String,
    status: &'static str,
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
        .with_env_filter(default_log_filter())
        .init();

    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let data_dir = PathBuf::from(env::var("DATA_DIR").unwrap_or_else(|_| "./data".into()));
    let database_dir = PathBuf::from(
        env::var("DATABASE_DIR").unwrap_or_else(|_| data_dir.to_string_lossy().into_owned()),
    );
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
        .expect("restore database snapshot");
    let (key, generated) = load_or_create_key(&data_dir.join("quarterly-ready.key"))
        .await
        .expect("load encryption key");
    let database_url = format!("sqlite://{}?mode=rwc", database_path.display());
    let db = SqlitePoolOptions::new()
        // The deployed database is a single SQLite file on the mounted Azure
        // Files share. One connection avoids self-contention and makes a
        // rolling revision hand-off safe.
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
    migrate_with_retry(&db)
        .await
        .expect("run database migrations");
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
    let hmrc_integration = approved_integration_from_env();
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
        persistence: Arc::new(Mutex::new(())),
        limits: Arc::new(Mutex::new(HashMap::new())),
        client,
        billing_base_url: env::var("SOCIOBOT_BILLING_URL")
            .unwrap_or_else(|_| "https://api.sociobot.in/api/v1".into()),
        hmrc_integration,
        safe_qa_fixtures,
    };
    let integration_configured = state.hmrc_integration.is_some();
    let app = build_router(state, frontend_dir);

    info!(
        port,
        build_sha = BUILD_SHA,
        encryption_key = if generated { "generated" } else { "persisted" },
        hmrc_integration = if integration_configured {
            "configured"
        } else {
            "not_configured"
        },
        safe_qa_fixtures,
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
        .route("/api/hmrc/submit", post(submit_to_hmrc))
        .route("/api/qa/entitlement", get(safe_qa_entitlement))
        .route("/api/page-view", post(page_view))
        .route_service("/", get_service(ServeFile::new(index.clone())))
        .route_service("/demo", get_service(ServeFile::new(index.clone())))
        .route_service("/records", get_service(ServeFile::new(index.clone())))
        .route_service("/privacy", get_service(ServeFile::new(index.clone())))
        .route_service("/terms", get_service(ServeFile::new(index.clone())))
        .route_service("/share/:token", get_service(ServeFile::new(index)))
        .fallback_service(
            ServeDir::new(frontend_dir.clone())
                .not_found_service(ServeFile::new(frontend_dir.join("404.html"))),
        )
        .layer(DefaultBodyLimit::max(MAX_DOCUMENT_BYTES))
        .layer(middleware::from_fn(security_headers))
        .layer(middleware::from_fn_with_state(state.clone(), rate_limit))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> Json<Health<'static>> {
    Json(Health {
        status: "ok",
        build_sha: BUILD_SHA,
        safe_qa_fixtures: state.safe_qa_fixtures,
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
    let _persistence = state.persistence.lock().await;
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
    persist_database_snapshot(&state.database_path, &state.snapshot_path)
        .await
        .map_err(internal)?;
    Ok(Json(json!({ "saved": true, "updated_at": now })))
}

async fn create_share(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<(StatusCode, Json<ShareResult>), ApiError> {
    let _persistence = state.persistence.lock().await;
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
    persist_database_snapshot(&state.database_path, &state.snapshot_path)
        .await
        .map_err(internal)?;
    Ok((StatusCode::CREATED, Json(ShareResult { token, expires_at })))
}

async fn submit_to_hmrc(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<Json<SubmissionResult>, ApiError> {
    let _persistence = state.persistence.lock().await;
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
    if safe_fixture_authorized(state.safe_qa_fixtures, &licence, &input.document) {
        let submission_id = format!("safe-fixture-no-filing-{}", unix_now());
        write_audit(
            &state,
            &id,
            "safe_fixture_submission_checked",
            submission_id.as_bytes(),
        )
        .await?;
        persist_database_snapshot(&state.database_path, &state.snapshot_path)
            .await
            .map_err(internal)?;
        return Ok(Json(SubmissionResult {
            submission_id,
            status: "fixture_only_no_filing",
        }));
    }
    verify_licence_token(&state, &licence).await?;
    let integration = state.hmrc_integration.as_ref().ok_or(ApiError(
        StatusCode::SERVICE_UNAVAILABLE,
        "An approved HMRC integration is not configured for this service. Download the accountant pack or try again later.",
    ))?;
    let response = state
        .client
        .post(&integration.url)
        .bearer_auth(&integration.token)
        .header(
            "x-quarterly-ready-submission",
            "mtd-itsa-periodic-update-v1",
        )
        .json(&payload)
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
    let response: Value = response.json().await.map_err(|_| ApiError(
        StatusCode::BAD_GATEWAY,
        "The approved HMRC integration did not return a submission reference. No submission was made.",
    ))?;
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
    write_audit(
        &state,
        &id,
        "hmrc_submission_requested",
        submission_id.as_bytes(),
    )
    .await?;
    persist_database_snapshot(&state.database_path, &state.snapshot_path)
        .await
        .map_err(internal)?;
    Ok(Json(SubmissionResult {
        submission_id,
        status: "accepted",
    }))
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
    let url = env::var("HMRC_INTEGRATION_URL").ok()?;
    let token = env::var("HMRC_INTEGRATION_TOKEN").ok()?;
    if url.starts_with("https://") && !token.trim().is_empty() {
        Some(ApprovedIntegration { url, token })
    } else {
        None
    }
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
    if !request.uri().path().starts_with("/api/") {
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

async fn restore_database_snapshot(
    snapshot: &FsPath,
    database: &FsPath,
) -> Result<(), std::io::Error> {
    if fs::metadata(snapshot).await.is_ok() && fs::metadata(database).await.is_err() {
        fs::copy(snapshot, database).await?;
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
    // Azure Files supports overwrite copies but not the POSIX rename used for
    // an atomic replace. All real mutations are serialized by `persistence`,
    // so a direct overwrite is a consistent snapshot for this one replica.
    let mut source = fs::File::open(database).await?;
    let mut destination = fs::File::create(snapshot).await?;
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

    #[test]
    fn startup_migration_retries_transient_sqlite_locks() {
        assert!(is_database_locked_message("database is locked"));
        assert!(is_database_locked_message("SQLITE_BUSY: database is busy"));
        assert!(!is_database_locked_message("no such table"));
    }

    #[tokio::test]
    async fn workspace_snapshot_is_restored_after_a_restart() {
        let root =
            std::env::temp_dir().join(format!("quarterly-ready-snapshot-{}", Uuid::new_v4()));
        let durable = root.join("durable");
        let local = root.join("local");
        fs::create_dir_all(&durable).await.unwrap();
        fs::create_dir_all(&local).await.unwrap();
        let database = local.join("quarterly-ready.sqlite3");
        let snapshot = durable.join("quarterly-ready.snapshot.sqlite3");
        fs::write(&database, b"encrypted workspace snapshot")
            .await
            .unwrap();
        persist_database_snapshot(&database, &snapshot)
            .await
            .unwrap();
        fs::remove_file(&database).await.unwrap();
        restore_database_snapshot(&snapshot, &database)
            .await
            .unwrap();
        assert_eq!(
            fs::read(&database).await.unwrap(),
            b"encrypted workspace snapshot"
        );
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
            persistence: Arc::new(Mutex::new(())),
            limits: Arc::new(Mutex::new(HashMap::new())),
            client: reqwest::Client::new(),
            billing_base_url: "https://api.sociobot.in/api/v1".into(),
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
            persistence: Arc::new(Mutex::new(())),
            limits: Arc::new(Mutex::new(HashMap::new())),
            client: reqwest::Client::new(),
            billing_base_url: format!("http://{address}"),
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
            persistence: Arc::new(Mutex::new(())),
            limits: Arc::new(Mutex::new(HashMap::new())),
            client: reqwest::Client::new(),
            billing_base_url: format!("http://{address}"),
            hmrc_integration: Some(ApprovedIntegration {
                url: format!("http://{address}/submit"),
                token: "bridge-secret".into(),
            }),
            safe_qa_fixtures: false,
        };
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
        assert!(request_rx
            .recv()
            .await
            .unwrap()
            .starts_with("GET /products/mtd-quarterly-ready/verify?license=active-subscription"));
        let integration_request = request_rx.recv().await.unwrap();
        assert!(integration_request.starts_with("POST /submit"));
        assert!(integration_request.contains("quarterly-ready-mtd-itsa-periodic-update-v1"));
    }
}
