use serde::Serialize;
use uuid::Uuid;
use crate::server::Server;

#[derive(Default, Serialize)]
pub struct MSErrorJson {
    success: bool,
    error: String,
}

pub fn ms_error_format(str: impl Into<String>) -> String {
    match serde_json::to_string(&MSErrorJson{
        success: false, 
        error: str.into()}
    ) {
        Ok(str) => {str},
        Err(_) => { String::from("") }
    }
}

#[derive(Serialize)]
pub struct ServerResponce {
    success: bool,
    server: Server,
}

pub fn ms_return_server(server: Server) -> String {
    match serde_json::to_string(
        &ServerResponce{
        success: true, 
        server
    }
    ) {
        Ok(str) => {str},
        Err(_) => { ms_error_format("Failed to create server responce") }
    }
}

#[derive(Serialize)]
pub struct ServerTokenResponce {
    success: bool,
    token: Uuid,
}

pub fn ms_token_responce(token: Uuid) -> String {
    match serde_json::to_string(
        &ServerTokenResponce{
            success: true,
            token
        }
    ) {
        Ok(res) => {res},
        Err(_) => {ms_error_format("Failed to serialize server token")}
    }
}

pub fn ms_success_responce() -> &'static str {
    "{ \"success\": true }"
}