use crate::MASTER_SERVER;
use std::{
    sync::atomic::Ordering,
    time::{SystemTime, UNIX_EPOCH}
};

use shared::responses::{ms_return_server, ms_error_format};
use actix_web::{error::{self}, post, Error, HttpResponse, web};
use debug_print::debug_println;
use serde::{Serialize, Deserialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Serialize)]
struct ServerResponseJson<'a> {
    success: bool,
    servers: &'a serde_json::Value,
}

#[derive(Deserialize)]
pub struct JsonStruct {
    pub token: Uuid,
}

#[post("")]
pub async fn list_servers() -> Result<HttpResponse, Error> {
    debug_println!("list called");

    let servers = MASTER_SERVER.server_list.get_public_servers();

    let tme = SystemTime::now().duration_since(UNIX_EPOCH);

    let time = match tme {
        Ok(tme) => tme.as_secs(),
        Err(_) => {
            eprint!("Failed to get current system timestamp");
            0
        }
    };

    let mut valid_servers = Vec::new();
    let server_response_json_array: serde_json::Value;

    //Scope this to release the read lock as quick as we can
    {
        let servers = servers.read();

        for server in servers.iter() {
            if time != 0 && server.server_expiry_time < time {
                MASTER_SERVER
                    .server_list
                    .scrub_list
                    .store(true, Ordering::Relaxed);
                continue;
            }
            valid_servers.push(server);
        }
        server_response_json_array = json!(valid_servers);
    }

    let response = ServerResponseJson {
        success: true,
        servers: &server_response_json_array,
    };

    let json = match serde_json::to_string(&response) {
        Err(err) => {
            println!("Failed to serialise server list request json {}", err);
            return Err(error::ErrorInternalServerError(ms_error_format(
                "Failed to build response json",
            )));
        }
        Ok(json) => {json},
    };

    Ok(HttpResponse::Ok().body(json))
}

#[post("/byToken")]
pub async fn get_server_by_token(token: web::Json<JsonStruct>) -> Result<HttpResponse, Error> {
    debug_println!("server/byToken called");
    
/*
    //This is a spelling mistake caused by reading discord while programming, it will remain forever :D
    let tonken = match serde_json::from_slice::<JsonStruct>(&body){
*/

    let server = match MASTER_SERVER.server_list.get_hidden_server(token.token) {
        Some(server) => server,
        None => { return Err(error::ErrorNotFound(ms_error_format("Server not found")));  }
    };

    Ok(HttpResponse::Ok().body( ms_return_server(server)))
}