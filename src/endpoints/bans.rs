use crate::database::{check_identifier, BanInfo};
use actix_web::{post, web, Error, HttpResponse};
use shared::{
    responses::ms_is_banned_response,
    responses::{ms_bulk_check_response, BanIdentifiers},
};
use tracing::info_span;

#[post("/isBanned")]
async fn is_banned(is_banned_request: web::Json<BanIdentifiers>) -> Result<HttpResponse, Error> {
    let span = info_span!(
        "/isBanned ",
        uid = is_banned_request.0.id,
        ip = is_banned_request.0.ip
    );
    let _span = span.enter();

    match check_identifier(&is_banned_request.0).await {
        BanInfo::Banned(reason) => {
            Ok(HttpResponse::Ok().body(ms_is_banned_response(true, Some(reason))))
        }
        BanInfo::NotBanned => Ok(HttpResponse::Ok().body(ms_is_banned_response(false, None))),
    }
}

#[post("/bulkCheck")]
async fn bulk_check(players: web::Json<Vec<BanIdentifiers>>) -> Result<HttpResponse, Error> {
    let mut ban_vector: Vec<BanIdentifiers> = Vec::with_capacity(players.len());

    for mut player in players.0 {
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
