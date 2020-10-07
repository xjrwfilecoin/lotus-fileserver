#[macro_use]
extern crate lazy_static;

use app::Application;
mod app;
mod auth_middleware;
mod handler;
mod form;
mod model;
mod service;
mod dao;
mod db;

pub struct App {}

impl App {
    pub fn run() {
        let result = Application::run();
        match result {
            Ok(res) => {
                println!("app ok {:?}", res)
            }
            Err(err) => {
                println!("app error {:?}", err)
            }
            _ => {}
        }
    }
}