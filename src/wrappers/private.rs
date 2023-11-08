use shared::server::Server;
use std::net::SocketAddr;
use std::time::Duration;

use secret_sauce::validation::connection_validator;
use secret_sauce::validation::values_validator;

pub fn init() {
    secret_sauce::init();
}

pub fn validate_server_values(server: &Server) -> Result<bool, String> {
    match values_validator::server_values_check(server) {
        Ok(_) => Ok(true),
        Err(err) => Err(err),
    }
}

pub fn validate_server_connection(addr: SocketAddr, key: &str, timeout: Duration) -> Option<bool> {
    use shared::ms_config::get_global_config;
    let validator = connection_validator::new(addr, key, timeout)?;
    Some(
        validator
            .validate_server_connection(get_global_config().server_conn_validation_retry_count),
    )
}

pub fn red_endpoints(cfg: &mut actix_web::web::ServiceConfig) {
    secret_sauce::register_endpoints(cfg);
}
