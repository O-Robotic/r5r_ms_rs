use std::net::SocketAddr;
use std::time::Duration;
use shared::server::Server;

pub fn validate_server_values(_: &Server) -> Result<bool, String> {
    Ok(true)
}

pub fn validate_server_connection(_: SocketAddr, _: &str, _: Duration) -> Option<bool> {
    Some(true)
}

pub fn red_endpoints(_: &mut actix_web::web::ServiceConfig)  {
} 