use std::time::Duration;

use bytes::Bytes;
use futures::{try_join, StreamExt};
use rand::{Rng, RngCore};
use remoc::{
    chmux,
    codec::JsonCodec,
    rsync::{remote, RemoteSend},
};
use tokio::time::timeout;

use crate::loop_transport;

async fn loop_channel<T>() -> (
    (remote::Sender<T, JsonCodec>, remote::Receiver<T, JsonCodec>),
    (remote::Sender<T, JsonCodec>, remote::Receiver<T, JsonCodec>),
)
where
    T: RemoteSend,
{
    let cfg = chmux::Cfg::default();
    loop_transport!(0, transport_a_tx, transport_a_rx, transport_b_tx, transport_b_rx);
    try_join!(
        remoc::connect_framed(&cfg, transport_a_tx, transport_a_rx),
        remoc::connect_framed(&cfg, transport_b_tx, transport_b_rx),
    )
    .expect("creating remote loop channel failed")
}

#[tokio::test]
async fn negation() {
    crate::init();
    let ((mut a_tx, mut a_rx), (mut b_tx, mut b_rx)) = loop_channel::<i32>().await;

    let reply_task = tokio::spawn(async move {
        while let Some(i) = b_rx.recv().await.unwrap() {
            match b_tx.send(-i).await {
                Ok(()) => (),
                Err(err) if err.is_closed() => break,
                Err(err) => panic!("reply sending failed: {}", err),
            }
        }
    });

    for i in 1..1024 {
        println!("Sending {}", i);
        a_tx.send(i).await.unwrap();
        let r = a_rx.recv().await.unwrap().unwrap();
        println!("Received {}", r);
        assert_eq!(i, -r, "wrong reply");
    }
    drop(a_tx);

    reply_task.await.expect("reply task failed");
}

#[tokio::test]
async fn big_msg() {
    crate::init();
    let ((mut a_tx, mut a_rx), (mut b_tx, mut b_rx)) = loop_channel::<Vec<u8>>().await;

    let reply_task = tokio::spawn(async move {
        while let Some(mut msg) = b_rx.recv().await.unwrap() {
            msg.reverse();
            match b_tx.send(msg).await {
                Ok(()) => (),
                Err(err) if err.is_closed() => break,
                Err(err) => panic!("reply sending failed: {}", err),
            }
        }
    });

    let mut rng = rand::thread_rng();
    for _ in 1..10 {
        let size = rng.gen_range(0..1_000_000);
        let mut data: Vec<u8> = Vec::new();
        data.resize(size, 0);
        rng.fill_bytes(&mut data);

        let data_send = data.clone();
        println!("Sending message of length {}", data.len());
        a_tx.send(data_send).await.unwrap();
        let mut data_recv = a_rx.recv().await.unwrap().unwrap();
        println!("Received reply of length {}", data_recv.len());
        data_recv.reverse();
        assert_eq!(data, data_recv, "wrong reply");
    }
    drop(a_tx);

    reply_task.await.expect("reply task failed");
}

#[tokio::test]
async fn close_notify() {
    crate::init();
    let ((mut a_tx, _), (_, mut b_rx)) = loop_channel::<u8>().await;

    println!("Checking that channel is not closed");
    assert!(!a_tx.is_closed());
    let close_notify = a_tx.closed();
    if timeout(Duration::from_secs(1), close_notify).await.is_ok() {
        panic!("close notification before closure");
    }

    println!("Closing channel");
    b_rx.close().await;

    println!("Waiting for close notification");
    a_tx.closed().await;
    assert!(a_tx.is_closed());

    println!("Testing if send fails");
    if a_tx.send(1).await.is_ok() {
        panic!("send succeeded after closure");
    }
}
