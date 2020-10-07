use actix_web::{HttpResponse, web, Responder};

use crate::form::form::{SectorNotifyForm};
use crate::model::model::Status;
use crate::service::sector_service::SectorService;
use log::info;

pub async fn ping_handler(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(format!("pong: lotus-file-client receive ping \n --(0_0)-- \n"))
}


pub async fn sector_notify_handler(item: web::Json<SectorNotifyForm>) -> HttpResponse {
    println!("xjrw:{:?}", &item);
    // todo 传输文件
    let word_service = SectorService::new();
    match word_service {
        Ok(service) => {
            let result = service.add(&item.filename, &item.sector_id);
            match result {
                Ok(_res) => {
                    HttpResponse::Ok().json(Status::success())
                }
                Err(err) => {
                    HttpResponse::Ok().json(Status::fail_with_msg(err))
                }
            }
        }
        Err(err) => {
            HttpResponse::Ok().json(Status::fail_with_msg(err))
        }
    }
}






