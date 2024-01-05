use {
    crate::{
        database::{ban_identifier, search_for_ban},
        endpoints::panel::{get_mod_panel_js, get_ms_post_js},
        get_master_server,
    },
    actix_web::{error, get, post, web, Error, HttpResponse},
    maud::{html, Markup, PreEscaped, DOCTYPE},
    serde::Deserialize,
    shared::utils::format_identifier,
};

#[derive(Deserialize)]
pub struct BanRequest {
    pub identifier: String,
    pub reason: String,
    pub unban_timestamp: Option<u64>,
}

#[derive(Deserialize)]
pub struct BanSearchRequest {
    pub identifier: String,
}

#[derive(Deserialize)]
pub struct UnbanRequest {
    pub key: i32,
}

#[derive(Deserialize)]
pub struct KickFromServer {
    pub server_uid: String,
    pub player_uids: Vec<u64>,
}

#[post("/ban")]
async fn ban(ban_info: web::Json<BanRequest>) -> Result<HttpResponse, Error> {
    if ban_info.0.identifier.is_empty() {
        return Err(error::ErrorBadRequest("No identifier specified"));
    }

    let identifier = match format_identifier(ban_info.0.identifier) {
        Some(identifier) => identifier,
        None => return Err(error::ErrorBadRequest("Invalid Identifier")),
    };

    match ban_identifier(identifier, ban_info.0.reason, ban_info.0.unban_timestamp).await {
        Ok(result) => {
            if result {
                Ok(HttpResponse::Ok().finish())
            } else {
                Err(error::ErrorInternalServerError("Failed to ban identifier"))
            }
        },
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

#[post("/ban_search")]
async fn ban_search(ban_info: web::Json<BanSearchRequest>) -> Result<HttpResponse, Error> {
    let cleaned_identifier = match format_identifier(ban_info.0.identifier) {
        Some(identifier) => identifier,
        None => return Err(error::ErrorBadRequest("Invalid Identifier")),
    };
    let bans = search_for_ban(cleaned_identifier).await;
    Ok(HttpResponse::Ok().json(bans))
}

#[post("/unban")]
async fn unban_request(request: web::Json<UnbanRequest>) -> Result<HttpResponse, Error> {
    match crate::database::unban(request.0.key).await {
        true => Ok(HttpResponse::Ok().finish()),
        false => Err(error::ErrorInternalServerError("")),
    }
}

#[post("/kick")]
async fn kick_from_server(request: web::Json<KickFromServer>) -> Result<HttpResponse, Error> {
    if get_master_server()
        .server_list
        .update_kick_list(request.0.server_uid, request.0.player_uids)
    {
        Ok(HttpResponse::Ok().finish())
    } else {
        Err(error::ErrorInternalServerError("Could not find server"))
    }
}

#[get("/moderation/player")]
async fn moderation_panel() -> actix_web::Result<Markup> {
    Ok(html! {
        (DOCTYPE)
        meta name="viewport" content="width=device-width, initial-scale=1.0";
        html lang = "en" {
            (PreEscaped(get_ms_post_js()))
            (PreEscaped(get_mod_panel_js()))
            (PreEscaped(r#"<style>
            form { 
                display: table;
                margin: 2px;    
            }
            p { 
                display: table-row;
                margin: 2px;
            }
                
            input, label { 
                display: table-cell;
                margin: 2px;
            }
            
            table, th, td {
                border: 1px solid black;
                border-collapse: collapse;
            }
            
            th, td {
                padding-top: 5px;
                padding-bottom: 5px;
                padding-left: 10px;
                padding-right: 10px;
            }
    
        </style>"#))

            title {"ID Management"}

            body {
                h1 {"Identifier Management"}

                h2 {"Ban identifier"}

                form {

                    p {
                        label for = "ban_identifier" {"Identifier"}
                        input type = "text" id = "ban_identifier";
                    }

                    p {
                        label for = "ban_unbandate" {"Unban date (blank for permanent)"}
                        input type = "date" id = "ban_unbandate";
                    }

                    p {
                        label for = "ban_reason" {"Reason"};
                        input type = "text" id = "ban_reason";
                    }

                    p {
                        input type = "button" id = "ban_button" value = "Ban" onclick = "ban(this)" ;
                        p id = "ban_result";
                    }

                }

                h2 {"Ban search"}

                form {
                    p {
                        label for = "identifier" {"Identifier"}
                        input type = "text" id = "identifier";
                    }

                    p {
                        input type = "button" id = "ban_search_button" value = "Search" onclick = "check_identifier(this)";
                    }

                }

                table {
                    tr {
                        th {"Identifier"}
                        th {"Banned On"}
                        th {"Unban Date"}
                        th {"Reason"}
                    }
                    tbody id = "ban_list_table_body";
                }

            }
        }
    })
}
