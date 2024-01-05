use {
    actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware},
    actix_web::{
        self,
        cookie::{Key, SameSite},
        App, HttpServer,
    },
    database::init_postgres_pool,
    once_cell::sync::OnceCell,
    rustls::{Certificate, PrivateKey, ServerConfig},
    shared::ms_config::{Config, GLOBAL_CONFIG},
    sqlx::Postgres,
    std::{fs::File, io::BufReader, sync::Arc},
    tracing::Level,
};

pub mod database;
pub mod endpoints;
pub mod middleware;
pub mod server_list;
pub mod wrappers;

pub struct MasterServer {
    postgres_pool: Option<sqlx::Pool<Postgres>>,
    server_list: Arc<server_list::ServerList>,
}

impl MasterServer {
    pub async fn new() -> MasterServer {
        MasterServer {
            postgres_pool: init_postgres_pool().await,
            server_list: server_list::ServerList::new(),
        }
    }
}

pub static MASTER_SERVER: once_cell::sync::OnceCell<MasterServer> = OnceCell::new();

pub fn get_master_server() -> &'static MasterServer {
    MASTER_SERVER.get().unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    //Log to file as well at somepoint
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();

    //I still hate this
    if GLOBAL_CONFIG.set(Config::from_file()).is_err() {
        panic!("Could not create config")
    }

    if MASTER_SERVER.set(MasterServer::new().await).is_err() {
        panic!("Could not create masterserver data");
    }

    let cert_file = &mut BufReader::new(File::open("cert.pem").expect("Could not read cert file"));
    let key_file = &mut BufReader::new(File::open("key.pem").expect("Could not read key file"));

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

    //This is literally only used for a single thing, is probably a way to do this in lib itself
    wrappers::init();

    HttpServer::new(|| {
        let session_store =
            SessionMiddleware::builder(CookieSessionStore::default(), Key::generate())
                .cookie_secure(true)
                .cookie_content_security(actix_session::config::CookieContentSecurity::Private)
                .session_lifecycle(
                    PersistentSession::default()
                        .session_ttl(actix_web::cookie::time::Duration::minutes(10)),
                )
                .cookie_same_site(SameSite::Strict)
                .build();

        App::new()
            .wrap(session_store)
            .service(endpoints::eula::get_eula)
            .configure(endpoints::servers::servers_routes)
            .configure(endpoints::bans::ban_routes)
            .configure(endpoints::panel::panel_routes)
            .configure(wrappers::red_endpoints)
    })
    //Maybe allow https to be toggled for testing
    .bind_rustls_021("127.0.0.1:443", config.unwrap())?
    .run()
    .await
}
