use crate::server::Server;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct BanIdentifiers {
    pub id: Option<u64>,

    #[serde(skip_serializing)]
    pub ip: Option<String>,

    pub reason: Option<String>,
}

#[derive(Default, Serialize)]
struct ServerResponse {
    success: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    server: Option<Server>,

    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<Uuid>,

    #[serde(skip_serializing_if = "Option::is_none")]
    banned: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "bannedPlayers")]
    banned_players: Option<Vec<BanIdentifiers>>,
}

pub fn ms_error_format(str: impl Into<String>) -> String {
    match serde_json::to_string(&ServerResponse {
        success: false,
        error: Some(str.into()),
        ..Default::default()
    }) {
        Ok(str) => str,
        Err(_) => String::from(""),
    }
}

pub fn ms_return_server(server: &Server) -> String {
    match serde_json::to_string(&ServerResponse {
        success: true,
        server: Some(server.clone()),
        ..Default::default()
    }) {
        Ok(str) => str,
        Err(_) => ms_error_format("Failed to create server response"),
    }
}

pub fn ms_bulk_check_response(identifiers: Vec<BanIdentifiers>) -> String {
    match serde_json::to_string(&ServerResponse {
        success: true,
        banned_players: Some(identifiers),
        ..Default::default()
    }) {
        Ok(str) => str,
        Err(_) => ms_error_format("Failed to serialize bulk check response"),
    }
}

pub fn ms_is_banned_response(banned: bool, reason: Option<String>) -> String {
    let reason = reason.unwrap_or_else(|| String::from("You have been banned!"));

    match serde_json::to_string(&ServerResponse {
        success: true,
        banned: Some(banned),
        reason: Some(reason),
        ..Default::default()
    }) {
        Ok(str) => str,
        Err(_) => ms_error_format("Failed to serialize ban response"),
    }
}

pub fn ms_success_response() -> &'static str {
    "{ \"success\": true }"
}
