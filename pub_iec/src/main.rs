use std::env;
//Serialization crates
use serde::{Deserialize, Serialize};
use serde_json::to_vec;

// Crates that's handle time - If it is needed timezone, dates, gregorian calendar should look at chrono crate.
use std::time::Duration;
use std::time::Instant; // To measure time between a piece of the code
/* Use Case of std::time::Instant
let begin = Instant::now();
let time_reception = begin.elapsed();
println!("Duration of the time between begin and time_reception {:?}", time_reception);
*/
use tokio::time::sleep;

// Crates that deal with ethernet frames
use pnet::datalink::{self, Config};
use pnet::datalink::Channel::Ethernet;
use pnet::packet::ethernet::{EtherType, MutableEthernetPacket, EthernetPacket};

//Crate that's generate a checksum
use crc32fast::hash as crc32;

// Crate that's deal with serialization and deserialization regarding ASN1 TAG/LENGTH
// yasna
// def-parser
// asn1

// Const values defined in the Standard IEC61850-9-2
const TPID: u16 = 0x8100; // TPID for SV in IEC61850-9-2
const TCI: u16 = 0x8000; // TCI for SV in IEC61850-9-2
const ETHER_TYPE: u16 = 0x88BA; // EtherType for SV in IEC61850-9-2


// Declaration of Structs to build a SV Packet
#[derive(Debug, Clone)]
pub struct EthernetFrame
{
    pub destination:    [u8; 6],
    pub source:         [u8; 6],
    pub tpid:           [u16],
    pub tci:            [u16],
    pub ethertype:      [u16],
    pub payload:        SvPDU,
    pub fcs:            [u8; 4],
}
#[derive(Debug, Clone)]
pub struct SvPDU
{
    pub appid:          [u8; 2],
    pub length:         [u8; 2],
    pub reserved1:      [u8; 2],
    pub reserved2:      [u8; 2],
    pub apdu: SmvData,
     // pub padding:
    //The padding is used to guarantee the ethernet packet has more than 46 bytes to comply with the standard of Ethernet frame packets


}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SmvData
{
    pub sav_pdu_asn:    [u8; 2],
    pub no_asdu_asn:    [u8; 2],
    pub no_asdu:        [u8; 1],
    pub seq_asdu_asn:   [u8; 2],
    pub asdu_asn:       [u8;2],
    pub sv_id_asn:      [u8; 2],
    pub sv_id:          [u32; 1],
    pub smp_cnt_asn:    [u8; 2],
    pub smp_cnt:        [u16; 1],
    pub conf_rev_asn:   [u8; 2],
    pub conf_rev:       [u32; 1],
    pub smp_synch_asn:  [u8; 2],
    pub smp_synch:      [u8; 1],
    pub seq_data:       [u8; 2],
    pub logical_node: LogicalNode,

}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogicalNode
{
    pub i_a:[i32],
    pub q_ia:[u32],
    pub i_b:[i32],
    pub q_ib:[u32],
    pub i_c:[i32],
    pub q_ic:[u32],
    pub i_n:[i32],
    pub q_in:[u32],
    pub v_a:[i32],
    pub q_va:[u32],
    pub v_b:[i32],
    pub q_vb:[u32],
    pub v_c:[i32],
    pub q_vc:[u32],
    pub v_n:[i32],
    pub q_vn:[u32],
}

// Implementation of the Impl(functions) regarding the structs



// The publisher function to send SV packets
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

        sleep(Duration::from_millis(5000)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let interface_name = env::args().nth(1).expect("Please provide an interface name as an argument");
    let publisher_task = tokio::spawn(publisher(interface_name));

    tokio::try_join!(publisher_task)?;

    Ok(())
}
