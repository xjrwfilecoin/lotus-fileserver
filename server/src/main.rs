use qfiletrans::start_server;
use std::thread;
use std::path::PathBuf;
use std::env;
use log::{info, error};

fn main() {
    println!("start qfiletrans server ...");
    let args: Vec<String> = env::args().collect();

    // let mut host = "0.0.0.0:8081";
    // let mut server_path = "/mnt/data/server";

    println!("{:?}", args);
    if args.len() < 2 {
        error!("please config host and server_path ...");
        return;
    }
    info!("server parameters {:?}", args);
    let t1 = thread::spawn(move || {
        start_server(args[1].as_str(), PathBuf::from(args[2].as_str()));
    });
    t1.join().unwrap();
}
