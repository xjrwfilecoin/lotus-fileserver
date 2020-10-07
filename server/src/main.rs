#[macro_use]
extern crate log;

use qfiletrans::start_server;
use std::thread;
use std::path::PathBuf;
use std::env;


fn main() {
    env_logger::init();
    info!("start qfiletrans server ...");
    let args: Vec<String> = env::args().collect();

    // let mut host = "0.0.0.0:8081";
    // let mut server_path = "/mnt/data/server";

    let work_path = env::var("WORKER_PATH").expect("please config WORKER_PATH env value ");

    if work_path == "" {
        error!("work path no find，please config env WORKER_PATH ");
        return;
    }

    if args.len() < 2 {
        error!("host address no find,please config host address");
        return;
    }
    info!("host {:?},worker_path {:?}", args[1], work_path);
    let t1 = thread::spawn(move || {
        start_server(args[1].as_str(), PathBuf::from(work_path));
    });
    t1.join().unwrap();
}
