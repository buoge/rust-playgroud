
use std::env;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {

    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8888".to_string());
    
    // conn to server
    let mut stream = TcpStream::connect(&addr).await?;

    // write command to server
    stream.write_all(b"gettime").await?;

    // waiting server response
    let mut buf: Vec<u8> = Vec::with_capacity(8128);

    let mut resp = [0u8;2048];
    loop {
        // 尝试一次读，返回到字节数组中
        let n = stream.read(&mut resp).await?;
        // 读取到的字节合并到buf中
        buf.extend_from_slice(&resp[0..n]);
        if n == 0 {
            // 流断开了
            panic!("Unexcepted EOF");
        } else if buf.len() >= 28 {
            // like: "Tue Oct 31 14:56:27 CST 2023" 时间格式 
            // buf 内容已经足够
            break;
        } else {
            // buf 中还没有足够内容，继续读取并填充
            continue;
        }
        // 转换并打印返回的信息
        let timeinfo = String::from_utf8(buf)?;
        println!("timeinfo is: {}",timeinfo);

        Ok(())
    }
}


