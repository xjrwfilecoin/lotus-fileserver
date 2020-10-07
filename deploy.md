#### P1-P2文件转移服务
说明
* 该程序是把p1的文件转移到p2机器上的独立文件服务
* 该程序部署到做p2服务的机器上面
* 先启动该程序，在启动 lotus-worker程序

编译安装

    // 下载代码 , 分支为 dev1 
    git clone https://github.com/xjrwfilecoin/lotus-fileserver.git
    
    // 进入项目目录
    cd lotus-fileserver
   
    // 编译
    cargo build --release --package server

    // 找到可执行文件
    ./target/release/server
    
    // 执行，参数为服务启动host,参数必须配置，
    ./server 0.0.0.0:8081 
        