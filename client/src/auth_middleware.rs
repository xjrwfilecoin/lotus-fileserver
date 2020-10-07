use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::{Error};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use futures::future::{Either, ok, Ready};

pub struct CheckLogin;

impl<S, B> Transform<S> for CheckLogin where
    S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CheckLoginMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CheckLoginMiddleware { service })
    }
}

pub struct CheckLoginMiddleware<S> {
    service: S,
}

impl<S, B> Service for CheckLoginMiddleware<S>
    where
        S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
        S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;
    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        if req.path() == "/login" {
            Either::Left(self.service.call(req))
        } else {
            let result = req.head().headers.get("Authorization");
            match result {
                Some(_token) => {
                    // todo
                    Either::Left(self.service.call(req))
                }
                None => {
                    // todo
                    // Either::Right(ok(req.into_response(
                    //     HttpResponse::Forbidden()
                    //         .finish()
                    //         .into_body(),
                    // )))
                    Either::Left(self.service.call(req))
                }
            }
        }
    }
}