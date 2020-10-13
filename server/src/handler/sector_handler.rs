use actix_web::{HttpResponse, web, Responder, HttpRequest};

use crate::form::form::{SectorNotifyForm};
use crate::model::model::Status;
use crate::service::sector_service::SectorService;
use log::info;
use actix_web::http::Error;
use actix_multipart::Multipart;
use futures::{TryStreamExt, StreamExt};
use std::io::Write;

pub async fn ping_handler(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(format!("pong: lotus-file-server receive ping \n --(0_0)-- \n"))
}



pub async fn save_file(req: HttpRequest,mut payload: Multipart) -> Result<HttpResponse, Error> {
    // iterate over multipart stream
    println!("{:?}",req);
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap();
        let filepath = format!("./tmp/{}", sanitize_filename::sanitize(&filename));

        // File::create is blocking operation, use threadpool
        let mut f = web::block(|| std::fs::File::create(filepath))
            .await
            .unwrap();

        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&data).map(|_| f)).await?;
        }
    }
    Ok(HttpResponse::Ok().body("success"))
}



pub async fn upload_file_handler(item: web::Json<SectorNotifyForm>) -> HttpResponse {
    println!("xjrw:{:?}", &item);
    // todo
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






