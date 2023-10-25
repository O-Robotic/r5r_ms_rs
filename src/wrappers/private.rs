use std::net::SocketAddr;
use std::time::Duration;
use shared::server::Server;

use secret_sauce::validation::connection_validator;
use secret_sauce::validation::values_validator;

pub fn validate_server_values(server: &Server) -> Result<bool, String> {
    match values_validator::server_values_check(server) {
        Ok(_) => { return Ok(true); },
        Err(err) => { return Err(err); }
    }
}

pub fn validate_server_connection(addr: SocketAddr, key: &str, timeout: Duration) -> Option<bool> {
    use shared::ms_config::GLOBAL_CONFIG;

    let validator = match connection_validator::new(addr, key, timeout) {
        Ok(validator) => validator,
        Err(_) => { return None }
    };

    Some(validator.validate_server_connection(GLOBAL_CONFIG.server_conn_validation_retry_count))
}

pub fn red_endpoints(cfg: &mut actix_web::web::ServiceConfig)  {
    secret_sauce::register_endpoints(cfg);
} 
