mod list;
mod post;

use actix_web::{
    self,
    web::{scope, ServiceConfig},
};

pub fn servers_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/servers")
            .service(post::post)
            .service(list::list_servers)
            .service(list::get_server_by_token)
    );
}
