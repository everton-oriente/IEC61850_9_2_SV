/*
trace!: The lowest level, used for very detailed debugging information.
debug!: Used for general debugging information.
info!: Used for informational messages that highlight the progress of the application at a high level.
warn!: Used for potentially harmful situations, warnings that indicate something might go wrong.
error!: Used for error events that might still allow the application to continue running.
fatal!: (Not commonly used in the log crate, but some logging systems have it) Indicates very severe error events that will presumably lead the application to abort.
When you set RUST_LOG=info, it includes logs from info!, warn!, and error! levels. Here’s a breakdown of what each setting would include:

RUST_LOG=trace: Logs everything (trace, debug, info, warn, error).
RUST_LOG=debug: Logs debug, info, warn, error.
RUST_LOG=info: Logs info, warn, error.
RUST_LOG=warn: Logs warn, error.
RUST_LOG=error: Logs only error.
*/


use std::env;
//Serialization crates
use serde::{Deserialize, Serialize};

// Crates that's handle time - If it is needed timezone, dates, gregorian calendar should look at chrono crate.
use std::time::Duration;
use std::time::Instant; // To measure time between a piece of the code

//crate that's create async threads
use tokio::time::sleep;

// Crates that deal with ethernet frames
use pnet::datalink::{self, Config, NetworkInterface};
use pnet::datalink::Channel::Ethernet;
use pnet::packet::ethernet::{EtherType, EthernetPacket};
use pnet::packet::Packet;

//Crate that's generate a checksum
//use crc32fast::hash as crc32;

// Crate that's deal with serialization and deserialization regarding ASN1 TAG/LENGTH
// yasna
// def-parser
// asn1

//Crate that's guarantee the usage of date and time
//use chrono::prelude::*;

//Crate that's handle Log
use log::{info, warn,  error};


// Const values defined in the Standard IEC61850-9-2
const TPID: u16 =       0x8100; // TPID for SV in IEC61850-9-2
const TCI: u16 =        0x8000; // TCI for SV in IEC61850-9-2
const ETHER_TYPE: u16 = 0x88BA; // EtherType for SV in IEC61850-9-2

// EthernetFrame structure definition (same as in publisher)
#[derive(Debug, Clone)]
pub struct EthernetFrame {
    pub destination: [u8; 6],
    pub source: [u8; 6],
    pub tpid: u16,
    pub tci: u16,
    pub ethertype: u16,
    pub payload: SvPDU,
    pub fcs: [u8; 4],
}

// SvPDU structure definition (same as in publisher)
#[derive(Debug, Clone)]
pub struct SvPDU {
    pub appid: [u8; 2],
    pub length: [u8; 2],
    pub reserved1: [u8; 2],
    pub reserved2: [u8; 2],
    pub apdu: SmvData,
}

// SmvData structure definition (same as in publisher)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SmvData {
    pub sav_pdu_asn: [u8; 2],
    pub no_asdu_asn: [u8; 2],
    pub no_asdu: u8,
    pub seq_asdu_asn: [u8; 2],
    pub asdu_asn: [u8; 2],
    pub sv_id_asn: [u8; 2],
    pub sv_id: [u32; 1],
    pub smp_cnt_asn: [u8; 2],
    pub smp_cnt: [u16; 1],
    pub conf_rev_asn: [u8; 2],
    pub conf_rev: [u32; 1],
    pub smp_synch_asn: [u8; 2],
    pub smp_synch: u8,
    pub seq_data: [u8; 2],
    pub logical_node: LogicalNode,
}

// LogicalNode structure definition (same as in publisher)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogicalNode {
    pub i_a: [i32; 1],
    pub q_ia: [u32; 1],
    pub i_b: [i32; 1],
    pub q_ib: [u32; 1],
    pub i_c: [i32; 1],
    pub q_ic: [u32; 1],
    pub i_n: [i32; 1],
    pub q_in: [u32; 1],
    pub v_a: [i32; 1],
    pub q_va: [u32; 1],
    pub v_b: [i32; 1],
    pub q_vb: [u32; 1],
    pub v_c: [i32; 1],
    pub q_vc: [u32; 1],
    pub v_n: [i32; 1],
    pub q_vn: [u32; 1],
}

impl EthernetFrame {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let destination = [bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]];
        let source = [bytes[6], bytes[7], bytes[8], bytes[9], bytes[10], bytes[11]];
        let tpid = u16::from_be_bytes([bytes[12], bytes[13]]);
        let tci = u16::from_be_bytes([bytes[14], bytes[15]]);
        let ethertype = u16::from_be_bytes([bytes[16], bytes[17]]);
        let payload = SvPDU::from_bytes(&bytes[18..bytes.len() - 4]);
        let fcs = [bytes[bytes.len() - 4], bytes[bytes.len() - 3], bytes[bytes.len() - 2], bytes[bytes.len() - 1]];

        Self {
            destination,
            source,
            tpid,
            tci,
            ethertype,
            payload,
            fcs,
        }
    }
}

impl SvPDU {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let appid = [bytes[0], bytes[1]];
        let length = [bytes[2], bytes[3]];
        let reserved1 = [bytes[4], bytes[5]];
        let reserved2 = [bytes[6], bytes[7]];
        let apdu = SmvData::from_bytes(&bytes[8..]);

        Self {
            appid,
            length,
            reserved1,
            reserved2,
            apdu,
        }
    }
}

impl SmvData {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let sav_pdu_asn = [bytes[0], bytes[1]];
        let no_asdu_asn = [bytes[2], bytes[3]];
        let no_asdu = bytes[4];
        let seq_asdu_asn = [bytes[5], bytes[6]];
        let asdu_asn = [bytes[7], bytes[8]];
        let sv_id_asn = [bytes[9], bytes[10]];
        let sv_id = [u32::from_be_bytes([bytes[11], bytes[12], bytes[13], bytes[14]])];
        let smp_cnt_asn = [bytes[15], bytes[16]];
        let smp_cnt = [u16::from_be_bytes([bytes[17], bytes[18]])];
        let conf_rev_asn = [bytes[19], bytes[20]];
        let conf_rev = [u32::from_be_bytes([bytes[21], bytes[22], bytes[23], bytes[24]])];
        let smp_synch_asn = [bytes[25], bytes[26]];
        let smp_synch = bytes[27];
        let seq_data = [bytes[28], bytes[29]];
        let logical_node = LogicalNode::from_bytes(&bytes[30..]);

        Self {
            sav_pdu_asn,
            no_asdu_asn,
            no_asdu,
            seq_asdu_asn,
            asdu_asn,
            sv_id_asn,
            sv_id,
            smp_cnt_asn,
            smp_cnt,
            conf_rev_asn,
            conf_rev,
            smp_synch_asn,
            smp_synch,
            seq_data,
            logical_node,
        }
    }
}

impl LogicalNode {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let i_a = [i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])];
        let q_ia = [u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]])];
        let i_b = [i32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]])];
        let q_ib = [u32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]])];
        let i_c = [i32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]])];
        let q_ic = [u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]])];
        let i_n = [i32::from_be_bytes([bytes[24], bytes[25], bytes[26], bytes[27]])];
        let q_in = [u32::from_be_bytes([bytes[28], bytes[29], bytes[30], bytes[31]])];
        let v_a = [i32::from_be_bytes([bytes[32], bytes[33], bytes[34], bytes[35]])];
        let q_va = [u32::from_be_bytes([bytes[36], bytes[37], bytes[38], bytes[39]])];
        let v_b = [i32::from_be_bytes([bytes[40], bytes[41], bytes[42], bytes[43]])];
        let q_vb = [u32::from_be_bytes([bytes[44], bytes[45], bytes[46], bytes[47]])];
        let v_c = [i32::from_be_bytes([bytes[48], bytes[49], bytes[50], bytes[51]])];
        let q_vc = [u32::from_be_bytes([bytes[52], bytes[53], bytes[54], bytes[55]])];
        let v_n = [i32::from_be_bytes([bytes[56], bytes[57], bytes[58], bytes[59]])];
        let q_vn = [u32::from_be_bytes([bytes[60], bytes[61], bytes[62], bytes[63]])];

        Self {
            i_a,
            q_ia,
            i_b,
            q_ib,
            i_c,
            q_ic,
            i_n,
            q_in,
            v_a,
            q_va,
            v_b,
            q_vb,
            v_c,
            q_vc,
            v_n,
            q_vn,
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize the Logger

    env_logger::init();


    let args: Vec<String> = env::args().collect();
    let interface_name = if args.len() > 1 {
        args[1].clone()
    } else {
        "eth0".to_string()
    };

    let interfaces = datalink::interfaces();
    let interface = interfaces.into_iter().find(|iface: &NetworkInterface| iface.name == interface_name).unwrap();

    let config = Config {
        read_timeout: Some(Duration::new(1, 0)),
        ..Default::default()
    };

    let (_tx, mut rx) = match datalink::channel(&interface, config) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e),
    };

    info!("Listening on interface {}", interface.name);
    loop {
        match rx.next() {
            Ok(frame) => {
                let packet = EthernetPacket::new(frame).unwrap();
                if packet.get_ethertype() == EtherType(TPID) || packet.get_ethertype() == EtherType(ETHER_TYPE) {
                    let ethernet_frame = EthernetFrame::from_bytes(packet.packet());
                    println!("Received Ethernet Frame: {:?}", ethernet_frame);  // Debug
                }
            }
            Err(e) => {
                println!("An error occurred while reading: {}", e);
            }
        }

        sleep(Duration::from_millis(400)).await;
    }
}
