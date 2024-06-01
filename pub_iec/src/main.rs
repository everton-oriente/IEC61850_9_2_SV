use tokio::net::UdpSocket;
use tokio::task;
use serde_json::to_vec;
use std::time::Duration;
use serde::{Serialize, Deserialize};

//Crates thats regarding about the ethernet frames
use pnet::datalink::{self, NetworkInterface};
use pnet::datalink::Channel::Ethernet;
use pnet::packet::{Packet, MutablePacket};
use pnet::packet::ethernet::{EthernetPacket, MutableEthernetPacket};


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

async fn publisher() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let socket = UdpSocket::bind("127.0.0.1:0").await?;
    socket.connect("127.0.0.1:4000").await?;

    loop {
        let sv = SampledValue {
            destination_mac: [0x01, 0x0C, 0xCD, 0x01, 0x00, 0x01], // Example MAC
            ethertype: 0x88BA, // Example Ethertype for IEC61850-9-2
            app_id: 0x4000,
            length: 8 + 8, // Example length (header + payload)
            sv_id: "SampleSV".to_string(),
            smp_cnt: 1235,
            smp_synch: true,
            // Initialize other fields
        };

        let sv_bytes = to_vec(&sv)?;
        socket.send(&sv_bytes).await?;
        tokio::time::sleep(Duration::from_millis(5000)).await; // Send every second
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let publisher_task = tokio::spawn(publisher());

    tokio::try_join!(publisher_task)?;

    Ok(())
}
