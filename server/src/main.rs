#[macro_use]
extern crate log;

use qfiletrans::start_server;
use std::thread;
use std::path::PathBuf;
use std::env;


fn main() {
    env_logger::init();

    info!("start qfiletrans server ...");

    let host = match env::args().nth(1){
        Some(h) => h,
        None=> "0.0.0.0:8081".to_owned()
    };

    let work_path = env::var("WORKER_PATH").expect("please config WORKER_PATH env value ");

    if work_path == "" {
        error!("work path no find，please config env WORKER_PATH ");
        return;
    }

    info!("worker_path {}, start on:{}", work_path,host);
    let t1 = thread::spawn(move || {
        start_server(host.as_str(),PathBuf::from(work_path));
    });
    t1.join().unwrap();
}
