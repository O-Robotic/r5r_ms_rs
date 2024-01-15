use {
    maud::{Markup, html, PreEscaped, DOCTYPE},
    actix_web::{error, get, web, Error, HttpResponse},
    shared::ms_config::{get_global_config, Config},
};

#[get("/get_config")]
pub async fn get_config() -> Result<HttpResponse, Error> {
    match serde_json::to_string(get_global_config()) {
        Ok(str) => Ok(HttpResponse::Ok().body(str)),
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

#[get("/set_config")]
pub async fn set_config(json: web::Json<Config>) -> Result<HttpResponse, Error> {
    let mut config = get_global_config();
    config.clone_from(&&json.0);

    Ok(HttpResponse::Ok().finish())
}

//WIP
#[get("/config")]
pub async fn panel_config() -> actix_web::Result<Markup> {

    let cfg = get_global_config();
    Ok(
        html! {
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            html lang = "en" {
                (DOCTYPE)
                title {"MS Config"}
                (PreEscaped(r#"<style>
                    #server_timeout {
                        width: 3em;
                    }

                    #server_conn_timeout {
                        width: 5em;
                    }

                    input[type="number"]::-webkit-outer-spin-button, 
                    input[type="number"]::-webkit-inner-spin-button {
                        -webkit-appearance: none;
                        margin: 0;
                    }
                    input[type="number"] {
                        -moz-appearance: textfield;
                    }

                    #sdk_list {
                        margin-top: 1em;
                        width: 12em;
                    }
                    
                    td {
                        height: 50px;
                        vertical-align: middle;
                    }

                    button {
                        margin-top: 5px;
                    }
                </style>
                "#))

                body {
                    h1 {"Configuration"}

                    table {
                        tr {
                            td {"Validate Server Connection"}
                            td { input type = "checkbox" id = "player_auth" checked = (cfg.validate_server_conn); }
                        }

                        tr {
                            td {"Server connection validation timeout (ms)"}
                            td { input type = "number" id = "server_conn_timeout" min = "0" step = "1" value = (cfg.server_conn_validation_listen_timeout); }
                        }

                        tr {
                            td {"Server connection validation retry count"}
                            td { input type = "number" id = "server_conn_retry_count" min = "0" step = "1" value = (cfg.server_conn_validation_retry_count); }
                        }

                        tr {
                            td {"Server Timeout"}
                            td { input type = "number" id = "server_timeout" step = "1" min = "0" value = (cfg.server_timeout); }
                        }
                    }
                }
            }
        }

    )

}