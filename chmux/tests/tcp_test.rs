use tokio::prelude::*;
use tokio::runtime::Runtime;
use tokio::net::TcpListener;
use tokio_util::codec::{FramedRead, FramedWrite};
use tokio_util::codec::length_delimited::LengthDelimitedCodec;
use tokio_serde::SymmetricallyFramed;
use tokio_serde::formats::SymmetricalJson;

use chmux;

fn tcp_server() {
    let mut rt = Runtime::new().unwrap();
    rt.block_on(async {

        let mut listener = TcpListener::bind("127.0.0.1:9887").await.unwrap();

        let (mut socket, _) = listener.accept().await.unwrap();
        let (socket_rx, socket_tx) = socket.split();
        let framed_tx = FramedWrite::new(socket_tx, LengthDelimitedCodec::new());
        let framed_rx = FramedRead::new(socket_rx, LengthDelimitedCodec::new());
        let msg_tx = SymmetricallyFramed::new(framed_tx, SymmetricalJson::default());
        let msg_rx = SymmetricallyFramed::new(framed_rx, SymmetricalJson::default());

        let (mux, _, mut server) = chmux::Multiplexer::new(chmux::Cfg::default(), msg_tx, msg_rx);

        loop {
            match server.next().await {
                Some((service, req)) => {
                    let (mut tx, mut rx) = req.accept().await.split();
                    tx.send("Hi".to_string()).await.unwrap();
                }
            }
        }
    });
}



fn tcp_client() {
    
}


#[test]
fn tcp_test() {

}
