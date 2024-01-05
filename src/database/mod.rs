use {
    crate::get_master_server,
    argon2::{
        password_hash::{
            rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
        },
        Argon2,
    },
    chrono::{NaiveDateTime, Utc},
    serde::Serialize,
    shared::{ms_config::get_global_config, responses::BanIdentifiers, utils::format_ip_to_ipv6},
    sqlx::{postgres::PgPoolOptions, types::chrono, Pool, Postgres},
    tracing::{debug, error, info},
};

pub enum BanInfo {
    Banned(String),
    NotBanned,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct BanRows {
    pub ban_id: i32,
    pub identifier: Option<String>,
    pub reason: Option<String>,
    pub banned_on: sqlx::types::chrono::DateTime<Utc>,
    pub unban_date: Option<sqlx::types::chrono::DateTime<Utc>>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct EULAData {
    pub version: i32,
    pub lang: String,
    pub contents: String,
}

#[derive(sqlx::FromRow)]
struct User {
    pw_hash: String,
}

pub async fn init_postgres_pool() -> Option<Pool<Postgres>> {
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
            error!("Failed to init db pool {}", err);
            None
        }
    }
}

pub async fn website_auth(username: &String, password: String) -> bool {
    if username.is_empty() || password.is_empty() {
        debug!("No ip or id provided as an identifier");
        return false;
    };

    let pool = match &get_master_server().postgres_pool {
        Some(pool) => pool,
        None => {
            error!("Could not get database pool");
            return false;
        }
    };

    let res = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(username.clone())
        .fetch_all(pool)
        .await;

    let user = match res {
        Ok(result) => result,
        Err(err) => {
            debug!("Error while processing user lookup: {}", err);
            return false;
        }
    };

    if user.is_empty() {
        debug!("Tried to auth '{}' but user didnt exist", username);
        return false;
    }

    if user[0].pw_hash == "placeholder" {
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);

        let pw_hash = match argon2.hash_password(password.as_bytes(), &salt) {
            Ok(hash) => hash,
            Err(_) => return false,
        };

        match sqlx::query("UPDATE users SET pw_hash = $1 WHERE username = $2")
            .bind(pw_hash.to_string())
            .bind(username)
            .execute(pool)
            .await
        {
            Ok(updated) => {
                if updated.rows_affected() != 0 {
                    return true;
                }
            }
            Err(err) => {
                debug!("Error when updating placeholder pw: {}", err);
                return false;
            }
        }
    }

    let hash = match PasswordHash::new(&user[0].pw_hash) {
        Ok(hash) => hash,
        Err(_) => return false,
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &hash)
        .is_ok()
}

pub async fn ban_identifier(identifier: String, reason: String, unban_date: Option<u64>) -> Result<bool, String> {
    let pool = match &get_master_server().postgres_pool {
        Some(pool) => pool,
        None => {
            let err_str = String::from("Could not get DB Pool");
            error!(err_str);
            return Err(err_str);
        }
    };

    let mut query = match unban_date {
        Some(_) => {
            let time = NaiveDateTime::from_timestamp_opt(unban_date.unwrap() as i64, 0).unwrap();
            let time = chrono::DateTime::<Utc>::from_utc(time, Utc);

            //Do not allow ban expiry in the past 
            if time < chrono::Utc::now() {
                debug!("Rejecting ban request for ban expiry being in the past");
                return Err(String::from("Ban expiry date is in the past"));
            }

            sqlx::query("INSERT INTO bans(unban_date, identifier, reason) VALUES ($1, $2, $3)")
                .bind(time)
        }
        None => sqlx::query("INSERT INTO bans(identifier, reason) VALUES ($1, $2)"),
    };

    query = query.bind(identifier);
    query = query.bind(reason);

    let result = query.execute(pool).await;

    match result {
        Ok(res) => Ok(res.rows_affected() != 0),
        Err(err) => {
            println!("err {}", err);
            Err(format!("Database Error: {}", err))
        }
    }
}

pub async fn check_identifier(identifiers: &BanIdentifiers) -> BanInfo {
    if identifiers.id.is_none() && identifiers.ip.is_none() {
        debug!("No ip or id provided as an identifier");
        return BanInfo::NotBanned;
    };

    let pool = match &get_master_server().postgres_pool {
        Some(pool) => pool,
        None => {
            error!("Could not get database pool");
            return BanInfo::NotBanned;
        }
    };

    debug!(
        "Checking ban identifiers: id: '{:?}' ip: '{:?}'",
        identifiers.id, identifiers.ip
    );

    let mut query_builder = sqlx::QueryBuilder::new("SELECT * FROM bans WHERE identifier IN (");

    let mut values = query_builder.separated(",");

    if identifiers.id.is_some() {
        values.push_bind(identifiers.id.unwrap_or(0).to_string());
    }
    if identifiers.ip.is_some() {
        let ip_str = format_ip_to_ipv6(identifiers.ip.clone().unwrap());
        values.push_bind(ip_str);
    }

    values.push_unseparated(")");

    let query = query_builder.build_query_as::<BanRows>();
    let response = query.fetch_all(pool).await;

    let mut rows = match response {
        Ok(response) => response,
        Err(err) => {
            error!("Error while performing sql request: {}", err);
            match get_global_config().ban_fail_condition {
                true => return BanInfo::Banned(String::from("An internal error occurred")),
                false => return BanInfo::NotBanned,
            }
        }
    };

    if rows.is_empty() {
        debug!("Request returned no rows, identifier is not banned");
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

    debug!("Identifier is not banned");

    BanInfo::NotBanned
}

pub async fn get_most_recent_bans(limit: u16) -> Option<Vec<BanRows>> {
    let pool = match &get_master_server().postgres_pool {
        Some(pool) => pool,
        None => {
            error!("Could not get database pool");
            return None;
        }
    };

    let query = sqlx::query_as::<_, BanRows>("SELECT * FROM bans ORDER BY DESC LIMIT $1")
        .bind(limit as i32)
        .fetch_all(pool)
        .await;

    match query {
        Ok(rows) => Some(rows),
        Err(_) => None,
    }
}

pub async fn search_for_ban(identifier: String) -> Option<Vec<BanRows>> {
    let pool = match &get_master_server().postgres_pool {
        Some(pool) => pool,
        None => {
            error!("Could not get database pool");
            return None;
        }
    };

    let query = sqlx::query_as::<_, BanRows>("SELECT * FROM bans WHERE identifier = $1")
        .bind(identifier)
        .fetch_all(pool)
        .await;

    match query {
        Ok(rows) => Some(rows),
        Err(_) => None,
    }
}

pub async fn unban(key: i32) -> bool {
    let pool = match &get_master_server().postgres_pool {
        Some(pool) => pool,
        None => {
            error!("Could not get database pool");
            return false;
        }
    };

    let response = sqlx::query("DELETE FROM bans WHERE ban_id = $1")
        .bind(key)
        .execute(pool)
        .await;

    match response {
        Ok(_) => true,
        Err(err) => {
            error!("Failed to ban player: {}", err);
            false
        }
    }
}

pub async fn get_latest_eula(language: String) -> Option<EULAData> {

    let pool = match &get_master_server().postgres_pool {
        Some(pool) => pool,
        None => {
            error!("Could not get database pool");
            return None;
        }
    };

    let response = sqlx::query_as::<_, EULAData>("SELECT version, lang, contents FROM eulas WHERE lang = $1 ORDER BY \"version\" DESC LIMIT 1")
    .bind(language)
    .fetch_one(pool).await;

    match response {
        Ok(eula) => Some(eula),
        Err(err) => {
            error!("Error while getting eula from db: {}", err);
            None
        }
    }
}