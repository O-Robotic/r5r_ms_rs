use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
    time::Duration,
};

use actix_web::{error, post, web, Error, HttpRequest, HttpResponse};

use crate::{get_master_server, wrappers};

use shared::ms_config::get_global_config;
use shared::responses::*;
use shared::server::Server;
use tracing::{error, span, Level};

#[post("/add")]
async fn post(req: HttpRequest, server: web::Json<Server>) -> Result<HttpResponse, Error> {
    let span = span!(
        Level::DEBUG,
        "/add ",
        name = server.0.name,
        key = server.0.key.to_string()
    );
    let _enter = span.enter();

    match wrappers::validate_server_values(&server) {
        //Ok == do nothing everything is chill
        Ok(_) => {}
        Err(error) => {
            let str = ms_error_format(error);
            println!("{}", str);
            return Err(error::ErrorBadRequest(str));
        }
    }

    let mut sock_adr = match req.peer_addr() {
        Some(adr) => adr,
        None => {
            error!("Actix peer_addr was None");
            SocketAddr::new(IpAddr::from_str("0.0.0.0").unwrap(), 0000)
        }
    };

    sock_adr.set_port(server.port);

    let duration =
        Duration::from_millis(get_global_config().server_conn_validation_listen_timeout as u64);
    let connection_valid: bool =
        wrappers::validate_server_connection(sock_adr, &server.key, duration).unwrap_or(false);

    if !connection_valid {
        return Err(error::ErrorNotAcceptable(ms_error_format(
            "Check your ports lol",
        )));
    };

    match get_master_server().server_list.add_server(server.0) {
        Some(server) => Ok(HttpResponse::Ok().body(ms_return_server(&server))),
        None => Err(error::ErrorInternalServerError(ms_error_format(
            "Failed to add server to server list",
        ))),
    }
}
