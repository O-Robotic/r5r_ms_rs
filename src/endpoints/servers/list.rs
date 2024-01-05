use {
    crate::get_master_server,
    actix_web::{
        error::{self},
        post, web, Error, HttpResponse,
    },
    serde::{Deserialize, Serialize},
    serde_json::json,
    shared::responses::{ms_error_format, ms_return_server},
    std::{
        sync::atomic::Ordering,
        time::{Duration, SystemTime, UNIX_EPOCH},
    },
    uuid::Uuid,
};

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
    let servers = get_master_server().server_list.get_public_servers();

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::new(0, 0))
        .as_secs();

    let mut valid_servers = Vec::new();
    let server_response_json_array: serde_json::Value;

    //Scope this to release the read lock as quick as we can
    {
        let servers = servers.read();

        for server in servers.iter() {
            if time != 0 && server.internal.server_expiry_time < time {
                get_master_server()
                    .server_list
                    .scrub_needed
                    .store(true, Ordering::Relaxed);
                continue;
            }
            valid_servers.push(&server.server);
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
        Ok(json) => json,
    };

    Ok(HttpResponse::Ok().body(json))
}

#[post("/byToken")]
pub async fn get_server_by_token(token: web::Json<JsonStruct>) -> Result<HttpResponse, Error> {
    /*
        //This is a spelling mistake caused by reading discord while programming, it will remain forever :D
        let tonken = match serde_json::from_slice::<JsonStruct>(&body){
    */

    let server = match get_master_server()
        .server_list
        .get_hidden_server(token.token)
    {
        Some(server) => server.server,
        None => {
            return Err(error::ErrorNotFound(ms_error_format("Server not found")));
        }
    };

    Ok(HttpResponse::Ok().body(ms_return_server(&server)))
}
