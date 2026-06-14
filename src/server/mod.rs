use actix_web::web;
mod auth_validation;
mod file_server;
mod premarket_getter_handler;
mod premarket_server;
mod premarket_validation;
mod proxy;
mod public_server;
mod user_server;
mod user_server_extractor;
mod vesing_server_handler;
mod whitelist_handlers;

pub fn init_servers(cfg: &mut web::ServiceConfig) {
    cfg.service(public_server::public_scope())
        .service(user_server::user_scope())
        .service(file_server::file_scope())
        // .service(premarket_server::private_scope())
        .service(premarket_server::pub_scope())
        .service(proxy::proxy_scope());
}
