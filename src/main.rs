use actix_web::{self, web, App, HttpServer};
use database::init_db_pool;
use once_cell::sync::OnceCell;
use rustls::{Certificate, PrivateKey, ServerConfig};
use shared::ms_config::{Config, GLOBAL_CONFIG};
use sqlx::Postgres;
use std::io::BufReader;
use std::{fs::File, sync::Arc};
use tracing::Level;

pub mod database;
pub mod endpoints;
pub mod server_list;
pub mod wrappers;

pub struct MasterServer {
    db_pool: Option<sqlx::Pool<Postgres>>,
    server_list: Arc<server_list::ServerList>,
}

impl MasterServer {
    pub async fn new() -> MasterServer {
        MasterServer {
            db_pool: init_db_pool().await,
            server_list: server_list::ServerList::new(),
        }
    }
}

static MASTER_SERVER: once_cell::sync::OnceCell<MasterServer> = OnceCell::new();

pub fn get_master_server() -> &'static MasterServer {
    MASTER_SERVER.get().unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();

    GLOBAL_CONFIG.set(Config::new());
    MASTER_SERVER.set(MasterServer::new().await);

    let cert_file = &mut BufReader::new(File::open("cert.pem")?);
    let key_file = &mut BufReader::new(File::open("key.pem")?);

    let cert = rustls_pemfile::certs(cert_file)?
        .into_iter()
        .map(Certificate)
        .collect();
    let key = PrivateKey(rustls_pemfile::pkcs8_private_keys(key_file)?.remove(0));

    let config = ServerConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(cert, key);

    wrappers::init();

    HttpServer::new(|| {
        App::new()
            .service(
                web::scope("/servers")
                    .service(endpoints::list::list_servers)
                    .service(endpoints::list::get_server_by_token)
                    .service(endpoints::post::post),
            )
            .service(
                web::scope("/banlist")
                    .service(endpoints::bans::is_banned)
                    .service(endpoints::bans::bulk_check),
            )
            .configure(wrappers::red_endpoints)
    })
    .bind_rustls("127.0.0.1:8080", config.unwrap())?
    .run()
    .await
}
