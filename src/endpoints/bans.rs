use actix_web::error;
use shared::responses::ms_error_format;

use {
    crate::{
        database::{check_identifier, BanInfo},
        get_master_server,
    },
    actix_web::{
        post, web,
        web::{scope, ServiceConfig},
        Error, HttpResponse,
    },
    serde::Deserialize,
    shared::{
        responses::ms_is_banned_response,
        responses::{ms_bulk_check_response, BanIdentifiers},
        server::Player,
    },
    tracing::info_span,
};

#[derive(Deserialize)]
pub struct BulkCheckRequest {
    pub uid: String,
    pub players: Vec<BanIdentifiers>,
}

pub fn ban_routes(cfg: &mut ServiceConfig) {
    cfg.service(scope("/banlist")
        .service(is_banned)
        .service(bulk_check)
    );
}

#[post("/isBanned")]
pub async fn is_banned(
    is_banned_request: web::Json<BanIdentifiers>,
) -> Result<HttpResponse, Error> {
    let span = info_span!(
        "/isBanned ",
        uid = is_banned_request.0.id,
        ip = is_banned_request.0.ip
    );
    let _span = span.enter();

    if is_banned_request.0.uid.is_none() || get_master_server().server_list.does_server_exist(is_banned_request.0.uid.as_ref().unwrap())
    {
        return Err(error::ErrorUnauthorized(ms_error_format("Unlisted Server")))
    }

    match check_identifier(&is_banned_request.0).await {
        BanInfo::Banned(reason) => {
            Ok(HttpResponse::Ok().body(ms_is_banned_response(true, Some(reason))))
        }
        BanInfo::NotBanned => Ok(HttpResponse::Ok().body(ms_is_banned_response(false, None))),
    }
}

#[post("/bulkCheck")]
pub async fn bulk_check(request: web::Json<BulkCheckRequest>) -> Result<HttpResponse, Error> {
    let mut ban_vector: Vec<BanIdentifiers> = Vec::with_capacity(request.0.players.len());
    let lists = get_master_server().server_list.clone();

    if !lists.does_server_exist(&request.0.uid)
    {
        return Err(error::ErrorUnauthorized(ms_error_format("Unlisted Server")))
    }

    for list in [&lists.hidden_servers, &lists.public_servers] {
        let mut list = list.upgradable_read();
        if let Some(index) = list
            .iter()
            .position(|server| server.internal.uid == request.uid)
        {
            let mut players: Vec<Player> = Vec::with_capacity(request.0.players.len());

            for player in &request.0.players {
                players.push(Player {
                    name: String::new(),
                    ip: player.ip.clone(),
                    uid: player.id,
                })
            }

            list.with_upgraded(|list| {
                list[index].players = players;
                for kick in list[index].kick_list.iter() {
                    ban_vector.push(BanIdentifiers {
                        uid: None,
                        id: Some(*kick),
                        ip: None,
                        reason: Some(String::from("Kicked from server")),
                    })
                }
                list[index].kick_list.clear();
            });

            break;
        }
    }

    for mut player in request.0.players {
        match check_identifier(&player).await {
            BanInfo::Banned(reason) => {
                player.reason = Some(reason);
                ban_vector.push(player);
            }
            BanInfo::NotBanned => {}
        }
    }

    Ok(HttpResponse::Ok().body(ms_bulk_check_response(ban_vector)))
}
