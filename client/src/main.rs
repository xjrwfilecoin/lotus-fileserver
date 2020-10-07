use qfiletrans::start_upload;
use std::thread;

fn main() {
    println!("start qfiletrans client ...");
    let t2 = thread::spawn(||{
        start_upload(String::from("localhost:8081"),&PathBuf::from("/mnt/ssd/bench/cache/s-t01000-1/sc-02-data-tree-c-1.dat"),&PathBuf::from("s-t01000-1/sc-02-data-tree-c-1.dat"));
    });
}
