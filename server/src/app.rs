use actix_web::{
    App, HttpServer, middleware, web,
};
use crate::auth_middleware;
use crate::handler::{sector_handler};

pub struct Application {}

impl Application {
    #[actix_rt::main]
    pub async fn run() -> std::io::Result<()> {
        std::env::set_var("RUST_LOG", "actix_web=info");
        env_logger::init();
        HttpServer::new(|| {
            App::new()
                .wrap(middleware::Logger::default())
                .wrap(auth_middleware::CheckLogin)
                .data(web::JsonConfig::default().limit(4096)) // <- limit size of the payload (global configuration)
                .service(web::resource("/ping").route(web::get().to(sector_handler::ping_handler)))
                .service(web::resource("/sectorNotify").route(web::post().to(sector_handler::sector_notify_handler)))
        })
            .bind("127.0.0.1:9999")?
            .run()
            .await
    }
}