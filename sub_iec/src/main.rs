use tokio::net::UdpSocket;
use tokio::task;
use bytes::BytesMut;
use serde_json::from_slice;
use serde::{Serialize, Deserialize};
use std::time::Instant;




//use pnet::datalink::{self, NetworkInterface};
//use pnet::datalink::Channel::Ethernet;
//use pnet::packet::{Packet, MutablePacket};
//use pnet::packet::ethernet::{EthernetPacket, MutableEthernetPacket};

#[derive(Serialize, Deserialize, Debug)]
struct SampledValue {
    destination_mac: [u8; 6],
    ethertype: u16,
    app_id: u16,
    length: u16,
    sv_id: String,
    smp_cnt: u32,
    smp_synch: bool,
    // Additional fields as required by IEC61850-9-2
}

async fn subscriber() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let socket = UdpSocket::bind("127.0.0.1:4000").await?;

    let mut buf = vec![0u8; 1024];

    loop {
        
        let (len, _addr) = socket.recv_from(&mut buf).await?;
        let begin = Instant::now();
        let sv: SampledValue = from_slice(&buf[..len])?;
        let time_reception = begin.elapsed();
        println!("Received SV: {:?} within: {:?}", sv, time_reception);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let subscriber_task = tokio::spawn(subscriber());

    tokio::try_join!(subscriber_task)?;

    Ok(())
}
