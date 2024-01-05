use {
    crate::{
        endpoints::panel::{get_server_management_js, get_ms_post_js, GENERIC_STYLE},
        get_master_server,
    },
    actix_web::{error, get, web},
    maud::{html, Markup, PreEscaped, DOCTYPE},
};

#[get("/management/server/{server_id}")]
pub async fn server_management(server_id: web::Path<String>) -> actix_web::Result<Markup> {
    let server_id = server_id.into_inner();

    if server_id.is_empty() {
        return Err(error::ErrorNotFound("No server specified"));
    }

    if let Some(server) = get_master_server()
        .server_list
        .find_server_from_uid(server_id)
    {
        Ok(html! {
            (DOCTYPE)
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            html lang = "en" {
                (GENERIC_STYLE)
                (PreEscaped(get_ms_post_js()))
                (PreEscaped(get_server_management_js()))
                title {"Server Management"}

                body  {
                    h1 {(format!("Server: {}", server.server.name))}
                    @if server.players.is_empty() {
                        h2 {"No players known"}
                    } @else {
                        table {
                            tr {
                                th {"IP"}
                                th {"UID"}
                                th {"Kick player"}
                            }

                            @for player in server.players.iter() {

                                tr {
                                    td { (player.ip.clone().unwrap_or_else(String::new)) }
                                    td { (player.uid.unwrap_or(0)) }

                                    @if player.uid.is_some() {
                                        td { input type = "checkbox" name = "players_to_kick" value = (player.uid.unwrap()); }
                                    }
                                }
                            }
                        }
                        button type = "button" value = (&server.internal.uid) onclick = "kick_button_pressed(this)" {"Kick Player(s)"}
                        p id = "kick_message" style = "margin-left: 5px;";
                    }
                }
            }
        })
    } else {
        Err(error::ErrorNotFound("Server not found"))
    }
}
