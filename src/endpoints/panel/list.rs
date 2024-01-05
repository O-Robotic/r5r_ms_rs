use {
    crate::{endpoints::panel::GENERIC_STYLE, get_master_server},
    actix_web::get,
    chrono::{DateTime, NaiveDateTime, Utc},
    maud::{html, Markup, PreEscaped, DOCTYPE},
    serde::Serialize,
    shared::server::ServerInfo,
    std::time::{SystemTime, UNIX_EPOCH},
};

#[derive(Serialize)]
pub struct ServerList<'a> {
    pub public: &'a Vec<ServerInfo>,
    pub hidden: &'a Vec<ServerInfo>,
}

#[get("/")]
pub async fn public_list() -> actix_web::Result<Markup> {
    let server_list = get_master_server().server_list.get_public_servers().read();
    Ok(html! {
        (DOCTYPE)
        meta name="viewport" content="width=device-width, initial-scale=1.0";
        html lang="en" {
            (GENERIC_STYLE)
            title {"R5R server list"}

            body {
                h1 {"Server List"}
                table {
                    tr {
                        th { "Server Name" }
                        th { "Map" }
                        th { "Playlist" }
                        th { "Players / Max Players" }
                        th { "Description" }
                    }

                    @for server_info in server_list.iter() {
                        tr{
                            td { (&server_info.server.name) }
                            td { (&server_info.server.map) }
                            td { (&server_info.server.playlist )}
                            td { (format!("{}/{}", server_info.server.player_count, server_info.server.max_players)) }
                            td { ( server_info.server.description.clone().unwrap_or_else(String::new))}
                        }
                    }
                }
            }
        }
    })
}

//This is the private list that will show hidden servers as well as public ones
#[get("/list")]
pub async fn private_list() -> actix_web::Result<Markup> {
    let pub_list = get_master_server().server_list.get_public_servers().read();
    let hidden_list = get_master_server().server_list.get_hidden_servers().read();
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Ok(html! {
        (DOCTYPE)
        meta name="viewport" content="width=device-width, initial-scale=1.0";
        html lang = "en" {
            (PreEscaped(r#"<script>
                function manage_server_pressed(e) {
                    let test = "/panel/management/server/" + encodeURIComponent(e.value);
                    window.location.href = test;        
                }
                </script>"#))
            (GENERIC_STYLE)
            title {"R5R Server List"}

            body {
                h1 {"Servers"}

                h2 {"Public Servers"}
                table {
                    tr {
                        th {"Name"}
                        th {"Map"}
                        th {"Playlist"}
                        th {"IP:Port"}
                        th {"Key"}
                        th {"Checksum"}
                        th {"Version"}
                        th {"Next post due"}
                        th {"Started at"}
                        th;
                    }

                    @for server in pub_list.iter() {
                        tr {
                            td {(&server.server.name)}
                            td {(&server.server.map)}
                            td {(&server.server.playlist)}
                            td {(format!("{}:{}", server.server.ip, server.server.port))}
                            td {(&server.server.key)}
                            td {(&server.server.checksum)}
                            td {(&server.server.version)}
                            td {((server.internal.server_expiry_time - current_time))}
                            td {({
                                let dt: DateTime<Utc> = chrono::DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt((server.internal.time_stamp / 1000) as i64, 0).unwrap(),Utc);
                                dt.to_rfc3339()
                            })}
                            td {button type = "button" value = (server.internal.uid) onclick = "manage_server_pressed(this)" { "Manage" }}
                        }
                    }
                }

                h2 {"Hidden Servers"}
                table {
                        tr {
                            th {"Name"}
                            th {"Map"}
                            th {"Playlist"}
                            th {"IP:Port"}
                            th {"Key"}
                            th {"Checksum"}
                            th {"Version"}
                            th {"Next post due"}
                            th {"Started at"}
                            th;
                        }

                        @for server in hidden_list.iter() {
                        tr {
                            td {(&server.server.name)}
                            td {(&server.server.map)}
                            td {(&server.server.playlist)}
                            td {(format!("{}:{}", server.server.ip, server.server.port))}
                            td {(&server.server.key)}
                            td {(&server.server.checksum)}
                            td {(&server.server.version)}
                            td {((server.internal.server_expiry_time - current_time))}
                            td {({
                                let dt: DateTime<Utc> = chrono::DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt((server.internal.time_stamp / 1000) as i64, 0).unwrap(),Utc);
                                dt.to_rfc3339()
                            })}
                            td {(server.internal.token.unwrap().to_string())}
                            td {button type = "button" value = (server.internal.uid) onclick = "manage_server_pressed(this)"  { "Manage" }}
                        }
                    }
                }
            }
        }
    })
}
