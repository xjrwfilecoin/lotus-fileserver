
#[test]
fn test01(){
    env_logger::init();
    log::trace!("init");
    for n in 1..16 {
        let x = 1 << n;
        log::info!("2^{}={}",n,x);
    }
}