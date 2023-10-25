use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr, time::Duration,
};

use actix_web::{error, post, web, Error, HttpRequest, HttpResponse};

use crate::{
    utils::ms_error_format,
    MASTER_SERVER, wrappers,
};

use shared::ms_config::GLOBAL_CONFIG;
use shared::server::Server;

#[post("/add")]
pub async fn post(req: HttpRequest, server: web::Json<Server>) -> Result<HttpResponse, Error> {
    
    match wrappers::validate_server_values(&server) {
        //Ok == do nothing everything is chill
        Ok(_) => {},
        Err(error) => {
            let str= ms_error_format(error);
            println!("{}", str);
            return Err(error::ErrorBadRequest(str));
        } 
    }

    let mut sock_adr = match req.peer_addr() {
        Some(adr) => adr,
        None => SocketAddr::new(IpAddr::from_str("0.0.0.0").unwrap(), 0000),
    };

    sock_adr.set_port(server.port);

    let duration = Duration::from_millis(u64::from(GLOBAL_CONFIG.server_conn_validation_listen_timeout));

    let connection_valid: bool = match wrappers::validate_server_connection(sock_adr, &server.key, duration) {
        Some(val) => val,
        None => {false},
    };

    if !connection_valid {
        return Err(error::ErrorNotAcceptable(ms_error_format("Check your ports lol")));
    };

    match MASTER_SERVER.server_list.add_server(server.0) {
        Some(token) => {
            return Ok(HttpResponse::Ok().body(format!("{{\"success\": true, \"token\": \"{}\" }}", token))); },
        None => { return Ok(HttpResponse::Ok().body("{\"success\": true}")); }
    }

}
