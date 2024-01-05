use {
    crate::server::{HostInfo, Player, Server, ServerInfo},
    serde::{Deserialize, Serialize},
    uuid::Uuid,
};

#[derive(Deserialize, Serialize)]
pub struct BanIdentifiers {
    pub uid: Option<String>,
    pub id: Option<u64>,
    #[serde(skip_serializing)]
    pub ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Default, Serialize)]
struct ServerResponse {
    success: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    server: Option<Server>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "server", flatten)]
    server_flatten: Option<Server>,

    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    server_info: Option<ServerInfo>,

    #[serde(skip_serializing_if = "Option::is_none")]
    players: Option<Vec<Player>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<Uuid>,

    #[serde(skip_serializing_if = "Option::is_none")]
    banned: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "bannedPlayers")]
    banned_players: Option<Vec<BanIdentifiers>>,

    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    host_info: Option<HostInfo>,

    #[serde(skip_serializing_if = "Option::is_none")]
    uid: Option<String>,

}

pub fn ms_error_format(str: impl Into<String>) -> String {
    match serde_json::to_string(&ServerResponse {
        success: false,
        error: Some(str.into()),
        ..Default::default()
    }) {
        Ok(str) => str,
        Err(_) => String::new(),
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

pub fn ms_return_server_info(server: &ServerInfo) -> String {
    match serde_json::to_string(&ServerResponse {
        success: true,
        server_info: Some(server.clone()),
        ..Default::default()
    }) {
        Ok(str) => str,
        Err(_) => ms_error_format("Failed to create server response"),
    }
}

pub fn ms_return_host_info(info: HostInfo) -> String {
    match serde_json::to_string(&ServerResponse {
        success: true,
        host_info: Some(info),
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
    let reason = reason.unwrap_or_else(|| {
        if banned {
            String::from("You have been banned!")
        } else {
            String::new()
        }
    });

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

pub fn ms_generic_response(key:String, data: String) -> String {
    format!("{{ \"success\": true, \"{}\": {} }}", key, data)
}

pub fn ms_success_response() -> &'static str {
    "{ \"success\": true }"
}
