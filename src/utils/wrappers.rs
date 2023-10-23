use std::net::SocketAddr;
use std::time::Duration;
use shared::server::Server;

#[cfg(feature = "secret_sauce")]
use secret_sauce::validation::connection_validator;
#[cfg(feature = "secret_sauce")]
use secret_sauce::validation::values_validator;

#[cfg(feature = "secret_sauce")]
pub fn validate_server_connection(addr: SocketAddr, key: &str, timeout: Duration) -> Option<bool> {
    use shared::ms_config::GLOBAL_CONFIG;

    let validator = match connection_validator::new(addr, key, timeout) {
        Ok(validator) => validator,
        Err(_) => { return None }
    };

    Some(validator.validate_server_connection(GLOBAL_CONFIG.server_conn_validation_retry_count))
}

#[cfg(feature = "secret_sauce")]
pub fn validate_server_values(server: &Server) -> Result<bool, String> {
    match values_validator::server_values_check(server) {
        Ok(_) => { return Ok(true); },
        Err(err) => { return Err(err); }
    }
}

#[cfg(not(feature = "secret_sauce"))]
pub fn validate_server_values(_: &Server) -> Result<bool, String> {
    Ok(true)
}

#[cfg(not(feature = "secret_sauce"))]
pub fn validate_server_connection(_: SocketAddr, _: &str, _: Duration) -> Option<bool> {
    Some(true)
}

pub fn ms_error_format(str: impl Into<String>) -> String {
    format!("{{\"success\":false,\"reason\":\"{}\" }}", str.into())
}

#[cfg(feature = "secret_sauce")]
pub fn red_endpoints(cfg: &mut actix_web::web::ServiceConfig)  {
    secret_sauce::register_endpoints(cfg);
} 

#[cfg(not(feature = "secret_sauce"))]
pub fn red_endpoints(_: &mut actix_web::web::ServiceConfig)  {
} 