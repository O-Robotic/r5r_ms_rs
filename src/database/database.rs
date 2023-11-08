use crate::get_master_server;
use chrono::Utc;
use shared::ms_config::get_global_config;
use shared::{responses::BanIdentifiers, utils::format_ip_to_ipv6};
use sqlx::Pool;
use sqlx::{postgres::PgPoolOptions, Postgres};
use tracing::{debug, info};

pub enum BanInfo {
    Banned(String),
    NotBanned,
}

#[derive(Debug, sqlx::FromRow)]
pub struct BanRows {
    pub reason: Option<String>,
    pub banned_on: sqlx::types::chrono::DateTime<Utc>,
    pub unban_date: Option<sqlx::types::chrono::DateTime<Utc>>,
}

pub async fn init_db_pool() -> Option<Pool<Postgres>> {
    if get_global_config().postgres_connection_uri.is_empty() {
        return None;
    };

    let pool = PgPoolOptions::new()
        .max_connections(15)
        .connect(&get_global_config().postgres_connection_uri)
        .await;

    match pool {
        Ok(pool) => Some(pool),
        Err(err) => {
            println!("Failed to init db pool {}", err);
            None
        }
    }
}

pub async fn check_identifier(identifiers: &BanIdentifiers) -> BanInfo {
    if identifiers.id.is_none() && identifiers.ip.is_none() {
        debug!("No ip or id provided as an identifier");
        return BanInfo::NotBanned;
    };

    let pool = match &get_master_server().db_pool {
        Some(pool) => pool,
        None => {
            debug!("Could not get database pool");
            return BanInfo::NotBanned;
        }
    };

    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT reason, banned_on, unban_date FROM bans WHERE identifier IN (",
    );

    let mut values = query_builder.separated(",");

    if identifiers.id.is_some() {
        values.push_bind(identifiers.id.unwrap_or_else(|| 0).to_string());
    }
    if identifiers.ip.is_some() {
        let ip_str = format_ip_to_ipv6(identifiers.ip.clone());
        values.push_bind(ip_str);
    }

    values.push_unseparated(")");

    let query = query_builder.build_query_as::<BanRows>();
    let response = query.fetch_all(pool).await;

    let mut rows = match response {
        Ok(response) => response,
        Err(err) => {
            debug!("Error while performing sql request: {}", err);
            match get_global_config().ban_fail_condition {
                true => return BanInfo::Banned(String::from("An internal error occurred")),
                false => return BanInfo::NotBanned,
            }
        }
    };

    if rows.is_empty() {
        info!("Request returned no rows, identifier is not banned");
        return BanInfo::NotBanned;
    }

    rows.sort_by(|a, b| b.banned_on.timestamp().cmp(&a.banned_on.timestamp()));

    let current_time = Utc::now();

    for i in rows {
        match i.unban_date {
            Some(time) => {
                if time.timestamp() != 0 && time.timestamp() < current_time.timestamp() {
                    info!("Identifier has an active ban, ban expiry time is '{}', current time is '{}'", 
                        time,
                        current_time.format("%d/%m/%Y %H:%M:%S")
                    );
                    return BanInfo::Banned(
                        i.reason
                            .unwrap_or_else(|| String::from("You have been banned!")),
                    );
                }
            }
            None => {
                info!(
                    "Identifier has an active ban with no expiry, banned on '{}'",
                    i.banned_on.format("%d/%m/%Y %H:%M:%S")
                );
                return BanInfo::Banned(
                    i.reason
                        .unwrap_or_else(|| String::from("You have been banned!")),
                );
            }
        }
    }

    info!("Identifier is not banned");

    BanInfo::NotBanned
}
