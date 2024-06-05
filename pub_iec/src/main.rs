use serde::{Deserialize, Serialize};
use serde_json::to_vec;
use std::env;
use std::time::Duration;
use tokio::time::sleep;

// Crates that deal with ethernet frames
use pnet::datalink::{self, Config};
use pnet::datalink::Channel::Ethernet;
use pnet::packet::ethernet::{EtherType, MutableEthernetPacket, EthernetPacket};

const ETHER_TYPE: u16 = 0x88BA; // EtherType for SV in IEC61850-9-2

#[derive(Serialize, Deserialize, Debug)]
struct SampledValue {
    sv_id: String,
    smp_cnt: u32,
    smp_synch: bool,
    // Additional fields as required by IEC61850-9-2
}

async fn publisher(interface_name: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let interface_name = interface_name.trim(); // Trim any whitespace
    println!("Looking for interface: '{}'", interface_name);

    let interfaces = datalink::interfaces();

    // Print all available interfaces for debugging purposes
    println!("Available network interfaces:");
    for iface in &interfaces {
        println!("Interface: {}, MAC: {:?}", iface.name, iface.mac);
    }

    let interface = interfaces.into_iter()
                              .find(|iface| iface.name == interface_name)
                              .expect("Failed to find the required interface");

    println!("Found interface: '{}'", interface.name);

    // Create a new channel, dealing with layer 2 packets
    let (mut tx, _rx) = match datalink::channel(&interface, Config::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
    };

    loop {
        let sv = SampledValue {
            sv_id: "SampleSV".to_string(),
            smp_cnt: 1235,
            smp_synch: true,
            // Initialize other fields
        };

        let sv_bytes = to_vec(&sv)?;
        let payload_len = sv_bytes.len();

        // Constructs and sends a single packet
        tx.build_and_send(1, EthernetPacket::minimum_packet_size() + payload_len, &mut |new_packet| {
            let mut ethernet_packet = MutableEthernetPacket::new(new_packet).unwrap();
            let mac = interface.mac.expect("Interface should have a MAC address");
            ethernet_packet.set_destination(mac);
            ethernet_packet.set_source(mac);
            ethernet_packet.set_ethertype(EtherType::new(ETHER_TYPE));
            ethernet_packet.set_payload(&sv_bytes);
        }).expect("Failed to send packet");

        println!("Message publish");

        sleep(Duration::from_millis(2000)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let interface_name = env::args().nth(1).expect("Please provide an interface name as an argument");
    let publisher_task = tokio::spawn(publisher(interface_name));

    tokio::try_join!(publisher_task)?;

    Ok(())
}
