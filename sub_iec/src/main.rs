use serde::{Deserialize, Serialize};
use serde_json::from_slice;
use std::env;
use std::time::Duration;
use tokio::time::sleep;

// Crates that deal with ethernet frames
use pnet::datalink::{self, Config, NetworkInterface};
use pnet::datalink::Channel::Ethernet;
use pnet::packet::ethernet::{EtherType, EthernetPacket};
use pnet::packet::Packet;

const ETHER_TYPE: u16 = 0x88BA; // EtherType for SV in IEC61850-9-2

#[derive(Serialize, Deserialize, Debug)]
struct SampledValue {
    sv_id: String,
    smp_cnt: u32,
    smp_synch: bool,
    // Additional fields as required by IEC61850-9-2
}

async fn subscriber(interface_name: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    let (_tx, mut rx) = match datalink::channel(&interface, Config::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
    };

    loop {
        match rx.next() {
            Ok(packet) => {
                let packet = EthernetPacket::new(packet).unwrap();
                
                // Check EtherType
                if packet.get_ethertype() == EtherType::new(ETHER_TYPE) {
                    let payload = packet.payload();
                    let sv: SampledValue = from_slice(payload).expect("Failed to deserialize payload");
                    println!("{:?}", sv);
                }
            },
            Err(e) => {
                // If an error occurs, we can handle it here
                eprintln!("An error occurred while reading: {}", e);
            }
        }

        sleep(Duration::from_millis(500)).await; // Polling interval
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let interface_name = env::args().nth(1).expect("Please provide an interface name as an argument");
    let subscriber_task = tokio::spawn(subscriber(interface_name));

    tokio::try_join!(subscriber_task)?;

    Ok(())
}
