use serde::Serialize;
use uuid::Uuid;
use crate::server::Server;

#[derive(Serialize)]
pub struct ServerResponse {
    success: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    server: Option<Server>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<Uuid>,
}

pub fn ms_error_format(str: impl Into<String>) -> String {
    match serde_json::to_string(&ServerResponse{
        success: false, 
        error: Some(str.into()),
        server: None,
        token: None,
    }
    ) {
        Ok(str) => {str},
        Err(_) => { String::from("") }
    }
}

pub fn ms_return_server(server: Server) -> String {
    match serde_json::to_string(
        &ServerResponse{
        success: true, 
        error: None,
        server: Some(server),
        token: None
    }
    ) {
        Ok(str) => {str},
        Err(_) => { ms_error_format("Failed to create server response") }
    }
}

pub fn ms_token_response(token: Uuid) -> String {
    match serde_json::to_string(
        &ServerResponse{
            success: true,
            error: None,
            server: None,
            token: Some(token),
        }
    ) {
        Ok(res) => {res},
        Err(_) => {ms_error_format("Failed to serialize server token")}
    }
}

pub fn ms_success_response() -> &'static str {
    "{ \"success\": true }"
}