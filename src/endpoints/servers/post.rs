use {
    crate::{get_master_server, wrappers},
    actix_web::{error, post, web, Error, HttpRequest, HttpResponse},
    shared::{
        ms_config::get_global_config,
        responses::{ms_error_format, ms_return_host_info},
        server::ServerWithUID,
    },
    std::time::Duration,
    tracing::{debug, error, span, Level},
};

#[post("/add")]
pub async fn post(
    req: HttpRequest,
    server: web::Json<ServerWithUID>,
) -> Result<HttpResponse, Error> {
    let span = span!(
        Level::DEBUG,
        "/add ",
        name = server.0.server.name,
        key = server.0.server.key.to_string(),
        uid = server.0.uid,
    );

    let _enter = span.enter();

    match wrappers::validate_server_values(&server.server) {
        Ok(_) => {}
        Err(error) => {
            debug!("Server field validation error: {}", error);
            return Err(error::ErrorBadRequest(ms_error_format(error)));
        }
    }

    let mut sock_adr = match req.peer_addr() {
        Some(adr) => adr,
        None => {
            error!("Actix peer_addr was None");
            return Err(error::ErrorInternalServerError(ms_error_format(
                "Failed to add server to server list",
            )));
        }
    };

    sock_adr.set_port(server.server.port);

    let duration =
        Duration::from_millis(get_global_config().server_conn_validation_listen_timeout as u64);

    let connection_valid: bool =
        wrappers::validate_server_connection(sock_adr, &server.server.key, duration)
            .unwrap_or(false);

    if !connection_valid {
        return Err(error::ErrorNotAcceptable(ms_error_format(
            "Unable to communicate, please forward your ports and check if the server is publicly accessible",
        )));
    };

    match get_master_server()
        .server_list
        .add_server(server.0, sock_adr)
        .await
    {
        Some(server) => Ok(HttpResponse::Ok().body(ms_return_host_info(server))),
        None => Err(error::ErrorInternalServerError(ms_error_format(
            "Failed to add server to server list",
        ))),
    }
}
