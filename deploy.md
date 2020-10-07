#### P1-P2文件转移服务
编译安装

    // 下载代码 , 分支为 dev1 
    git clone https://github.com/xjrwfilecoin/lotus-fileserver.git
    
    // 进入项目目录
    cd lotus-fileserver
   
    // 编译
    cargo build --release --package server

    // 找到可执行文件
    ./target/release/server
    
    // 执行，前者为服务启动host,参数必须配置，且顺序不能更改  
    ./server 0.0.0.0:8081 /mnt/data/server
        