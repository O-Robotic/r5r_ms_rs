#[macro_use]
extern crate lazy_static;

use actix_web::{App, HttpServer, web,self};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

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
    
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();

    builder.set_private_key_file("key.pem", SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();
    
    HttpServer::new(|| {
        App::new()
            .service(
                web::scope("/servers")
                .service(endpoints::list::list_servers)
                .service(endpoints::list::get_server_by_token)
                .service(endpoints::post::post)
            )
            
    })
    .bind_openssl("127.0.0.1:8080",builder)?
    .run()
    .await
}
