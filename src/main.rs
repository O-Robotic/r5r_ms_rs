#[macro_use]
extern crate lazy_static;


use std::fs::File;
use std::io::BufReader;
use actix_web::{App, HttpServer, web,self};
use rustls::{ServerConfig, Certificate, PrivateKey};
use utils::wrappers;

pub mod endpoints;
pub mod server_list;
pub mod utils;

pub struct MasterServer {
    server_list: server_list::ServerList,
}

impl MasterServer {
    pub fn new() -> Self {
        MasterServer {
            server_list: server_list::ServerList::default(),
        }
    }
}

//Should maybe use web_data for this
lazy_static! {
    pub static ref MASTER_SERVER: MasterServer = MasterServer::new();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {    
    let cert_file = &mut BufReader::new(File::open("cert.pem")?);
    let key_file = &mut BufReader::new(File::open("key.pem")?);

    let cert = rustls_pemfile::certs(cert_file)?.into_iter().map(Certificate).collect();
    let key = PrivateKey(rustls_pemfile::pkcs8_private_keys(key_file)?.remove(0));

    let config = ServerConfig::builder()
    .with_safe_default_cipher_suites()
    .with_safe_default_kx_groups()
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_no_client_auth()
    .with_single_cert(cert, key);
    
    HttpServer::new(|| {
        App::new()
            .service(
                web::scope("/servers")
                .service(endpoints::list::list_servers)
                .service(endpoints::list::get_server_by_token)
                .service(endpoints::post::post)
            ).configure(wrappers::red_endpoints)
    })
    .bind_rustls("127.0.0.1:8080", config.unwrap())?
    .run()
    .await
}
