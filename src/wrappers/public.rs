use {
    shared::server::Server,
    std::{net::SocketAddr, time::Duration},
};

pub fn init() {}

pub fn validate_server_values(_: &Server) -> Result<bool, String> {
    Ok(true)
}

pub fn validate_server_connection(_: SocketAddr, _: &str, _: Duration) -> Option<bool> {
    Some(true)
}

pub fn red_endpoints(_: &mut actix_web::web::ServiceConfig) {}
