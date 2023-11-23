use std::env;
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::process::Command;
use tokio_util::codec::{Framed,LengthDelimitedCodec};

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
        let (stream,_) = listener.accept().await?;
        // 包裹成 frame_steam
        let mut framed_stream = Framed::new(stream,LengthDelimitedCodec::new());

        // create a new task when client connect comming
        tokio::spawn( async move {
            while let Some(msg) = framed_stream.next().await {
                match msg {
                    Ok(msg) => {
                        let directive = String::from_utf8(msg.to_vec())
                        .expect("error when converting to string directive.");
                        print!("{directive}");
                        let output = process(&directive).await;
                        _ = framed_stream.send(Bytes::from(output)).await;

                    }
                    Err(e) => {
                        print!("{e:?}")
                    }
                }
            }
        });
    }
}

async fn process(directive: &str) -> String {
    if directive == "gettime" {
        let output = Command::new("date").output().await.unwrap();
        String::from_utf8(output.stdout).unwrap()
    } else {
        // 如果是其他指令返回无效
        "invalid command".to_owned()
    }
}
