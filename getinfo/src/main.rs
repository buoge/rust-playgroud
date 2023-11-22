use std::env;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::io::net::TcpListener;
use  tokio::io::process::Command;

/**
 *  1、在linux下有些命令这样使用ls -a（参数前一横）；
2、有些命令这样使用cp --help(参数前两横)；
3、还有一些这样使用tar -xzvf(参数前有一横)；
4、而有些这样使用tar xzvf(参数前没有横)。
关于命令的使用区别我们一一解释：
第一种：参数前有横的是 System V风格。
（1）参数用一横的说明后面的参数是字符形式。
（2）参数用两横的说明后面的参数是单词(有时-和--可以通用)形式。
第二种：参数前没有横的是 BSD风格。
有关System V和BSD的其他区别：
系统启动过程中 kernel 最后一步调用的是 init 程序，init 程序的执行有两种风格，即 System V 和 BSD。
System V 风格中 init 调用 /etc/inittab，BSD 风格调用 /etc/rc，它们的目的相同，都是根据 runlevel 执行一系列的程序。
     */
#[tokio::main]
async fn main() -> Result<(),Box<dyn std::error::Error>> {

    let addr = env::args().nth(1)
        .unwrap_or("127.0.0.1:8888".to_string());
    print!("===> listening on addr : {}",addr);
    let listener = TcpListener::bind(&addr).await?;

    // 无条件循环，表名自己始终处于服务状态
    loop {
        // waiting client connect
        let (mut socket,_) = listener.accept().await?;

        // create a new task when client connect comming
        tokio::spawn( async move {
            // 分配缓冲区
            let mut buf = [0;1024];
            let mut offset = 0;
            // 循环读取，不能确保一次能从网络读完全部数据
            // 正常情况下，读到数据就会返回，没有读到就会等待
            loop {
                let n = socket.read(&mut buf[offset..])
                .await
                .except("failed to read data from socket");
                // n 返回0的情况是碰到了 EOF，表明远端的写操作已经断开，这个要判断
                if n == 0 {
                    // eof end task
                    return;
                }
                print!("offset: {offset}, n:{n}");    
                let end = offset + n;

                if let Ok(directive) = std::str::from_utf8(&buf[..end]){
                    println!("{}", directive);
                    let output = process(directive).await;
                    println!("{output}");
                    socket.wtire_all(&output.as_bytes()).await
                    .except("failed to write data to socket");
                } else {
                    // 判断是否转换失败，如果是可能网络数据没有读取完成，要继续loop 然后读取
                    offset = end;
                }
            }
        });
    }
}

async fn process(directive: &str) -> String {
    if directive == "gettime" {
        let output = Command::new("date").output().await.unwarp();
        String::from_utf8(output.stdout).unwrap()
    } else {
        // 如果是其他指令返回无效
        "invalid command".to_owned()
    }
}
