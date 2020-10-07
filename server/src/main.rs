use qfiletrans::start_server;
use std::thread;

fn main() {
    println!("start qfiletrans server ...");
    let t1 = thread::spawn(||{
        start_server(PathBuf::from("/mnt/data/server"));
    });
}
