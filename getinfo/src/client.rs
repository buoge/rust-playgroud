use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use std::env;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8888".to_string());
    // conn to server
    let stream = TcpStream::connect(&addr).await?;
    let mut framed_stream = Framed::new(stream, LengthDelimitedCodec::new());

    framed_stream.send(Bytes::from("gettime")).await?;


    if let Some(msg) = framed_stream.next().await {
        match msg {
            Ok(msg) => {
                let timeinfo = String::from_utf8(msg.to_vec())?;
                println!("{}", timeinfo);
            },
            Err(e) => return Err(e.into())
        }
    }

    Ok(())
}


