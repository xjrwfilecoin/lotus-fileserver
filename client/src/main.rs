use qfiletrans::start_upload;
use std::thread;
use std::path::PathBuf;
fn main() {

    let host = std::env::args().nth(1).expect("host");
    let src = std::env::args().nth(2).expect("src");
    let dest = std::env::args().nth(3).expect("dest");

    println!("start upload {} to {}:/{}",src,host,dest);

    let t2 = thread::spawn(|| {
        start_upload(host, &PathBuf::from(src), &PathBuf::from(dest));
    });
    t2.join().unwrap()
}
