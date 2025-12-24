//! API route definitions
//!
//! # Endpoints
//!
//! ## Health
//! - `GET /health` - Health check
//!
//! ## Bots
//! - `GET /api/v1/bots` - List all bots
//! - `POST /api/v1/bots` - Add a new bot
//! - `GET /api/v1/bots/:id` - Get bot info
//! - `DELETE /api/v1/bots/:id` - Remove a bot
//! - `POST /api/v1/bots/:id/connect` - Connect bot to Switch
//! - `POST /api/v1/bots/:id/disconnect` - Disconnect bot from Switch
//!
//! ## Controls
//! - `POST /api/v1/bots/:id/click` - Click a button
//! - `POST /api/v1/bots/:id/press` - Press and hold a button
//! - `POST /api/v1/bots/:id/release` - Release a button
//! - `POST /api/v1/bots/:id/stick` - Set stick position
//!
//! ## Memory
//! - `POST /api/v1/bots/:id/peek` - Read memory
//! - `POST /api/v1/bots/:id/poke` - Write memory
//!
//! ## System
//! - `GET /api/v1/bots/:id/title` - Get current game title ID
//! - `GET /api/v1/bots/:id/version` - Get sys-botbase version

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::state::{AppState, BotInfo};

// ============================================================================
// Response Types
// ============================================================================

/// Generic API response wrapper
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

/// API response for health check
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
}

// ============================================================================
// Request Types
// ============================================================================

/// Request to add a new bot
#[derive(Deserialize)]
pub struct AddBotRequest {
    pub id: String,
    pub ip: String,
    pub port: u16,
    pub name: Option<String>,
}

/// Request to click/press/release a button
#[derive(Deserialize)]
pub struct ButtonRequest {
    pub button: String,
    #[serde(default)]
    pub delay_ms: Option<u64>,
}

/// Request to set stick position
#[derive(Deserialize)]
pub struct StickRequest {
    pub stick: String,
    pub x: i16,
    pub y: i16,
}

/// Request to read memory
#[derive(Deserialize)]
pub struct PeekRequest {
    pub offset: u64,
    pub size: usize,
    #[serde(default = "default_offset_type")]
    pub offset_type: String,
}

fn default_offset_type() -> String {
    "heap".to_string()
}

/// Request to write memory
#[derive(Deserialize)]
pub struct PokeRequest {
    pub offset: u64,
    pub data: String, // Hex-encoded data
    #[serde(default = "default_offset_type")]
    pub offset_type: String,
}

/// Response for memory read
#[derive(Serialize)]
pub struct PeekResponse {
    pub data: String, // Hex-encoded data
    pub size: usize,
}

/// Response for title ID
#[derive(Serialize)]
pub struct TitleResponse {
    pub title_id: String,
}

/// Response for version
#[derive(Serialize)]
pub struct VersionResponse {
    pub version: String,
}

// ============================================================================
// Handlers
// ============================================================================

/// Health check endpoint
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// List all bots
async fn list_bots(State(state): State<AppState>) -> Json<ApiResponse<Vec<BotInfo>>> {
    let bots = state.list_bots().await;
    Json(ApiResponse::success(bots))
}

/// Add a new bot
async fn add_bot(
    State(state): State<AppState>,
    Json(req): Json<AddBotRequest>,
) -> Result<Json<ApiResponse<BotInfo>>, (StatusCode, Json<ApiResponse<()>>)> {
    match state.add_bot(req.id, &req.ip, req.port, req.name).await {
        Ok(info) => Ok(Json(ApiResponse::success(info))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(e.to_string())),
        )),
    }
}

/// Get a specific bot
async fn get_bot(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<BotInfo>>, (StatusCode, Json<ApiResponse<()>>)> {
    match state.get_bot(&id).await {
        Some(bot_arc) => {
            let bot = bot_arc.read().await;
            Ok(Json(ApiResponse::success(bot.info.clone())))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Bot not found")),
        )),
    }
}

/// Remove a bot
async fn remove_bot(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<BotInfo>>, (StatusCode, Json<ApiResponse<()>>)> {
    match state.remove_bot(&id).await {
        Some(info) => Ok(Json(ApiResponse::success(info))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Bot not found")),
        )),
    }
}

/// Connect a bot to its Switch
async fn connect_bot(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<BotInfo>>, (StatusCode, Json<ApiResponse<()>>)> {
    match state.connect_bot(&id).await {
        Ok(info) => Ok(Json(ApiResponse::success(info))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        )),
    }
}

/// Disconnect a bot from its Switch
async fn disconnect_bot(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<BotInfo>>, (StatusCode, Json<ApiResponse<()>>)> {
    match state.disconnect_bot(&id).await {
        Ok(info) => Ok(Json(ApiResponse::success(info))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        )),
    }
}

/// Parse button string to SwitchButton
fn parse_button(s: &str) -> Option<sysbot_base::SwitchButton> {
    use sysbot_base::SwitchButton;
    match s.to_uppercase().as_str() {
        "A" => Some(SwitchButton::A),
        "B" => Some(SwitchButton::B),
        "X" => Some(SwitchButton::X),
        "Y" => Some(SwitchButton::Y),
        "L" => Some(SwitchButton::L),
        "R" => Some(SwitchButton::R),
        "ZL" => Some(SwitchButton::ZL),
        "ZR" => Some(SwitchButton::ZR),
        "PLUS" | "+" => Some(SwitchButton::Plus),
        "MINUS" | "-" => Some(SwitchButton::Minus),
        "DUP" | "UP" => Some(SwitchButton::DUp),
        "DDOWN" | "DOWN" => Some(SwitchButton::DDown),
        "DLEFT" | "LEFT" => Some(SwitchButton::DLeft),
        "DRIGHT" | "RIGHT" => Some(SwitchButton::DRight),
        "LSTICK" => Some(SwitchButton::LStick),
        "RSTICK" => Some(SwitchButton::RStick),
        "HOME" => Some(SwitchButton::Home),
        "CAPTURE" => Some(SwitchButton::Capture),
        _ => None,
    }
}

/// Parse stick string to SwitchStick
fn parse_stick(s: &str) -> Option<sysbot_base::SwitchStick> {
    use sysbot_base::SwitchStick;
    match s.to_uppercase().as_str() {
        "LEFT" | "L" => Some(SwitchStick::Left),
        "RIGHT" | "R" => Some(SwitchStick::Right),
        _ => None,
    }
}

/// Parse offset type string
fn parse_offset_type(s: &str) -> Option<sysbot_base::command::OffsetType> {
    use sysbot_base::command::OffsetType;
    match s.to_lowercase().as_str() {
        "heap" => Some(OffsetType::Heap),
        "main" => Some(OffsetType::Main),
        "absolute" | "abs" => Some(OffsetType::Absolute),
        _ => None,
    }
}

/// Click a button
async fn click_button(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ButtonRequest>,
) -> Result<Json<ApiResponse<&'static str>>, (StatusCode, Json<ApiResponse<()>>)> {
    let button = parse_button(&req.button).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(format!("Invalid button: {}", req.button))),
        )
    })?;

    let bot_arc = state.get_bot(&id).await.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Bot not found")),
        )
    })?;

    let mut bot = bot_arc.write().await;

    if let Some(delay) = req.delay_ms {
        bot.connection.click_with_delay(button, delay).await
    } else {
        bot.connection.click(button).await
    }
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        )
    })?;

    Ok(Json(ApiResponse::success("ok")))
}

/// Press and hold a button
async fn press_button(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ButtonRequest>,
) -> Result<Json<ApiResponse<&'static str>>, (StatusCode, Json<ApiResponse<()>>)> {
    let button = parse_button(&req.button).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(format!("Invalid button: {}", req.button))),
        )
    })?;

    let bot_arc = state.get_bot(&id).await.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Bot not found")),
        )
    })?;

    let mut bot = bot_arc.write().await;
    bot.connection.press(button).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        )
    })?;

    Ok(Json(ApiResponse::success("ok")))
}

/// Release a held button
async fn release_button(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ButtonRequest>,
) -> Result<Json<ApiResponse<&'static str>>, (StatusCode, Json<ApiResponse<()>>)> {
    let button = parse_button(&req.button).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(format!("Invalid button: {}", req.button))),
        )
    })?;

    let bot_arc = state.get_bot(&id).await.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Bot not found")),
        )
    })?;

    let mut bot = bot_arc.write().await;
    bot.connection.release(button).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        )
    })?;

    Ok(Json(ApiResponse::success("ok")))
}

/// Set stick position
async fn set_stick(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<StickRequest>,
) -> Result<Json<ApiResponse<&'static str>>, (StatusCode, Json<ApiResponse<()>>)> {
    let stick = parse_stick(&req.stick).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(format!("Invalid stick: {}", req.stick))),
        )
    })?;

    let bot_arc = state.get_bot(&id).await.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Bot not found")),
        )
    })?;

    let mut bot = bot_arc.write().await;
    bot.connection.set_stick(stick, req.x, req.y).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        )
    })?;

    Ok(Json(ApiResponse::success("ok")))
}

/// Read memory (peek)
async fn peek_memory(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<PeekRequest>,
) -> Result<Json<ApiResponse<PeekResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let offset_type = parse_offset_type(&req.offset_type).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(format!(
                "Invalid offset type: {}",
                req.offset_type
            ))),
        )
    })?;

    let bot_arc = state.get_bot(&id).await.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Bot not found")),
        )
    })?;

    let mut bot = bot_arc.write().await;
    let data = bot
        .connection
        .read_bytes(req.offset, req.size, offset_type)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(e.to_string())),
            )
        })?;

    let hex_data = data.iter().map(|b| format!("{:02X}", b)).collect::<String>();

    Ok(Json(ApiResponse::success(PeekResponse {
        data: hex_data,
        size: data.len(),
    })))
}

/// Write memory (poke)
async fn poke_memory(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<PokeRequest>,
) -> Result<Json<ApiResponse<&'static str>>, (StatusCode, Json<ApiResponse<()>>)> {
    let offset_type = parse_offset_type(&req.offset_type).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(format!(
                "Invalid offset type: {}",
                req.offset_type
            ))),
        )
    })?;

    // Decode hex data
    let data = sysbot_base::command::ResponseDecoder::decode_hex(req.data.as_bytes()).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(e.to_string())),
        )
    })?;

    let bot_arc = state.get_bot(&id).await.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Bot not found")),
        )
    })?;

    let mut bot = bot_arc.write().await;
    bot.connection
        .write_bytes(req.offset, &data, offset_type)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(e.to_string())),
            )
        })?;

    Ok(Json(ApiResponse::success("ok")))
}

/// Get current game title ID
async fn get_title_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<TitleResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let bot_arc = state.get_bot(&id).await.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Bot not found")),
        )
    })?;

    let mut bot = bot_arc.write().await;
    let title_id = bot.connection.get_title_id().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        )
    })?;

    Ok(Json(ApiResponse::success(TitleResponse {
        title_id: format!("0x{:016X}", title_id),
    })))
}

/// Get sys-botbase version
async fn get_version(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<VersionResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let bot_arc = state.get_bot(&id).await.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Bot not found")),
        )
    })?;

    let mut bot = bot_arc.write().await;
    let version = bot.connection.get_version().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        )
    })?;

    Ok(Json(ApiResponse::success(VersionResponse { version })))
}

// ============================================================================
// Router
// ============================================================================

/// Create the API router with all routes
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health
        .route("/health", get(health))
        // Bots CRUD
        .route("/api/v1/bots", get(list_bots).post(add_bot))
        .route("/api/v1/bots/{id}", get(get_bot).delete(remove_bot))
        // Bot connection
        .route("/api/v1/bots/{id}/connect", post(connect_bot))
        .route("/api/v1/bots/{id}/disconnect", post(disconnect_bot))
        // Controls
        .route("/api/v1/bots/{id}/click", post(click_button))
        .route("/api/v1/bots/{id}/press", post(press_button))
        .route("/api/v1/bots/{id}/release", post(release_button))
        .route("/api/v1/bots/{id}/stick", post(set_stick))
        // Memory
        .route("/api/v1/bots/{id}/peek", post(peek_memory))
        .route("/api/v1/bots/{id}/poke", post(poke_memory))
        // System
        .route("/api/v1/bots/{id}/title", get(get_title_id))
        .route("/api/v1/bots/{id}/version", get(get_version))
        .with_state(state)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
        let state = AppState::new();
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_bots_empty() {
        let state = AppState::new();
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/bots")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(json["success"].as_bool().unwrap());
        assert!(json["data"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_add_bot() {
        let state = AppState::new();
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/bots")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        r#"{"id": "bot1", "ip": "192.168.1.100", "port": 6000, "name": "TestBot"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(json["success"].as_bool().unwrap());
        assert_eq!(json["data"]["id"].as_str().unwrap(), "bot1");
        assert_eq!(json["data"]["name"].as_str().unwrap(), "TestBot");
    }

    #[tokio::test]
    async fn test_get_nonexistent_bot() {
        let state = AppState::new();
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/bots/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_parse_button() {
        assert!(parse_button("A").is_some());
        assert!(parse_button("a").is_some());
        assert!(parse_button("ZL").is_some());
        assert!(parse_button("PLUS").is_some());
        assert!(parse_button("+").is_some());
        assert!(parse_button("invalid").is_none());
    }

    #[test]
    fn test_parse_stick() {
        assert!(parse_stick("LEFT").is_some());
        assert!(parse_stick("L").is_some());
        assert!(parse_stick("right").is_some());
        assert!(parse_stick("invalid").is_none());
    }

    #[test]
    fn test_parse_offset_type() {
        assert!(parse_offset_type("heap").is_some());
        assert!(parse_offset_type("MAIN").is_some());
        assert!(parse_offset_type("abs").is_some());
        assert!(parse_offset_type("invalid").is_none());
    }
}
