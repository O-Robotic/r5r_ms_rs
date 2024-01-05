mod config;
mod list;
mod login;
mod main;
mod player_moderation;
mod server_management;
use maud::PreEscaped;

pub static GENERIC_STYLE: PreEscaped<&'static str> = PreEscaped(
    "
<style>
    table, th, td {
        border: 1px solid black;
        border-collapse: collapse
    }
    
    th, td {
        padding-top: 5px;
        padding-bottom: 5px;
        padding-left: 10px;
        padding-right: 10px;
    }

    button {
        margin-top: 5px;
    }
</style>
",
);

#[cfg(not(debug_assertions))]
static ID_MANAGEMENT_JS: &'static str = include_str!(r"../../javascript/identifier_management.js");

#[cfg(not(debug_assertions))]
static SERVER_MANAGEMENT_JS: &'static str = include_str!(r"../../javascript/server_management.js");

#[cfg(not(debug_assertions))]
static LOGIN_JS: &'static str = include_str!("../../javascript/login.js");

#[cfg(not(debug_assertions))]
static MS_POST_JS: &'static str = include_str!("../../javascript/ms_post.js");

#[cfg(not(debug_assertions))]
fn get_mod_panel_js() -> &'static str {
    ID_MANAGEMENT_JS
}

#[cfg(not(debug_assertions))]
fn get_server_management_js() -> &'static str {
    SERVER_MANAGEMENT_JS
}

#[cfg(not(debug_assertions))]
fn get_login_js() -> &'static str {
    LOGIN_JS
}

#[cfg(not(debug_assertions))]
fn get_ms_post_js() -> &'static str {
    MS_POST_JS
}

#[cfg(debug_assertions)]
fn get_mod_panel_js() -> String {
    std::fs::read_to_string("r5r_ms_rs\\src\\javascript\\identifier_management.js").unwrap()
}

#[cfg(debug_assertions)]
fn get_server_management_js() -> String {
    std::fs::read_to_string("r5r_ms_rs\\src\\javascript\\server_management.js").unwrap()
}

#[cfg(debug_assertions)]
fn get_login_js() -> String {
    std::fs::read_to_string("r5r_ms_rs\\src\\javascript\\login.js").unwrap()
}

#[cfg(debug_assertions)]
fn get_ms_post_js() -> String {
    std::fs::read_to_string("r5r_ms_rs\\src\\javascript\\ms_post.js").unwrap()
}

use actix_web::{
    self,
    web::{scope, ServiceConfig},
};
pub fn panel_routes(cfg: &mut ServiceConfig) {
    cfg.service(login::panel_auth)
        .service(list::public_list)
        .service(login::login_page)
        .service(
            scope("/panel")
                //.service(config::get_config)
                //.service(config::set_config)
                //.service(config::panel_config)
                .service(login::panel_logout)
                .service(player_moderation::ban)
                .service(player_moderation::ban_search)
                .service(player_moderation::unban_request)
                .service(player_moderation::kick_from_server)
                .service(main::panel_main_menu)
                .service(list::private_list)
                .service(server_management::server_management)
                .service(player_moderation::moderation_panel)
                .wrap(crate::middleware::auth::ProtectedEndpoint)
        );
}
