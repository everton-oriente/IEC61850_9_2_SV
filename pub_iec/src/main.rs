//for accessing environment variables.
use std::env;
use std::f32::consts::PI;
//Serialization crates
use serde::{Deserialize, Serialize};

// Crates that's handle time - If it is needed timezone, dates, gregorian calendar should look at chrono crate.
use std::time::Duration;
use std::time::Instant; // To measure time between a piece of the code

//crate that's create async threads
use tokio::time::sleep;

// Crates that deal with ethernet frames
use pnet::datalink::{self, Config};
use pnet::datalink::Channel::Ethernet;

//Crate that's generate a checksum
use crc32fast::hash as crc32;

// Crate that's deal with serialization and deserialization regarding ASN1 TAG/LENGTH
// yasna
// def-parser
// asn1

//Crate that's guarantee the usage of date and time
use chrono::prelude::*;

//Crate Logging
use log::{info, warn, error};

// Const values defined in the Standard IEC61850-9-2
const TPID: u16 =       0x8100; // TPID for SV in IEC61850-9-2
const TCI: u16 =        0x8000; // TCI for SV in IEC61850-9-2
const ETHER_TYPE: u16 = 0x88BA; // EtherType for SV in IEC61850-9-2
const FREQUENCY: f32 = 50.0; // Frequency of the system
const AMPLITUDE_VOLTAGE: f32 = 10000.0; // 10kV nominal voltage of the system
const AMPLITUDE_CURRENT: f32 = 1000.0; // 1kA nominal current of the systemprintln
const PHASE_A_RAD:f32 = 0.0;
const PHASE_B_RAD: f32 = 2.0943951023931953; // 120º degrees in radians
const PHASE_C_RAD: f32 = -2.0943951023931953; // -120º degrees in radians
const TWICE: f32 = 2.0000;

// Declaration of Structs to build a SV Packet

//EthernetFrame is regarding the Main SV Packet, Header, Payload and Checksum.
#[derive(Debug, Clone)]
pub struct EthernetFrame
{
    pub destination:    [u8; 6],
    pub source:         [u8; 6],
    pub tpid:           u16,
    pub tci:            u16,
    pub ethertype:      u16,
    pub payload:        SvPDU,
    pub fcs:            [u8; 4],
}

//SvPDU is regarding the part about the payload,
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

//SmvData is regarding the part is responsible for the data.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SmvData
{
    pub sav_pdu_asn:    [u8; 2],
    pub no_asdu_asn:    [u8; 2],
    pub no_asdu:        u8,
    pub seq_asdu_asn:   [u8; 2],
    pub asdu_asn:       [u8; 2],
    pub sv_id_asn:      [u8; 2],
    pub sv_id:          [u32; 1],
    pub smp_cnt_asn:    [u8; 2],
    pub smp_cnt:        [u16; 1],
    pub conf_rev_asn:   [u8; 2],
    pub conf_rev:       [u32; 1],
    pub smp_synch_asn:  [u8; 2],
    pub smp_synch:      u8,
    pub seq_data:       [u8; 2],
    pub logical_node: LogicalNode,

}

// This part is regarding of the DataSet
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogicalNode
{
    pub i_a:    [i32; 1],
    pub q_ia:   [u32; 1],
    pub i_b:    [i32; 1],
    pub q_ib:   [u32; 1],
    pub i_c:    [i32; 1],
    pub q_ic:   [u32; 1],
    pub i_n:    [i32; 1],
    pub q_in:   [u32; 1],
    pub v_a:    [i32; 1],
    pub q_va:   [u32; 1],
    pub v_b:    [i32; 1],
    pub q_vb:   [u32; 1],
    pub v_c:    [i32; 1],
    pub q_vc:   [u32; 1],
    pub v_n:    [i32; 1],
    pub q_vn:   [u32; 1],
}

// Implementation of functions regarding EthernetFrame struct
impl EthernetFrame {
    pub fn new(destination: [u8; 6], source: [u8; 6], tpid: u16, tci: u16, ethertype: u16, payload: SvPDU, fcs: [u8; 4]) -> Self {
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

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut frame = Vec::new();
        frame.extend(&self.destination);
        frame.extend(&self.source);
        frame.extend(&self.tpid.to_be_bytes());
        frame.extend(&self.tci.to_be_bytes());
        frame.extend(&self.ethertype.to_be_bytes());
        frame.extend(&self.payload.to_bytes());
        frame.extend(&self.fcs);
        frame
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let destination =   [bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]];
        let source =        [bytes[6], bytes[7], bytes[8], bytes[9], bytes[10], bytes[11]];
        let tpid =                u16::from_be_bytes([bytes[12], bytes[13]]);
        let tci =                 u16::from_be_bytes([bytes[14], bytes[15]]);
        let ethertype =           u16::from_be_bytes([bytes[16], bytes[17]]);
        let payload =             SvPDU::from_bytes(&bytes[18..bytes.len() - 4]);
        let fcs =           [bytes[bytes.len() - 4], bytes[bytes.len() - 3], bytes[bytes.len() - 2], bytes[bytes.len() - 1]];

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

// Implementation of functions regarding SvPDU struct
impl SvPDU {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut pdu = Vec::new();
        pdu.extend(&self.appid);
        pdu.extend(&self.length);
        pdu.extend(&self.reserved1);
        pdu.extend(&self.reserved2);
        pdu.extend(&self.apdu.to_bytes());
        pdu
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let appid =     [bytes[0], bytes[1]];
        let length =    [bytes[2], bytes[3]];
        let reserved1 = [bytes[4], bytes[5]];
        let reserved2 = [bytes[6], bytes[7]];
        let apdu =            SmvData::from_bytes(&bytes[8..]);

        Self {
            appid,
            length,
            reserved1,
            reserved2,
            apdu,
        }
    }
}
// Implementation of functions regarding SmvData struct
impl SmvData {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend(&self.sav_pdu_asn);
        data.extend(&self.no_asdu_asn);
        data.push(self.no_asdu);
        data.extend(&self.seq_asdu_asn);
        data.extend(&self.asdu_asn);
        data.extend(&self.sv_id_asn);
        for id in &self.sv_id {
            data.extend(&id.to_be_bytes());
        }
        data.extend(&self.smp_cnt_asn);
        for cnt in &self.smp_cnt {
            data.extend(&cnt.to_be_bytes());
        }
        data.extend(&self.conf_rev_asn);
        for rev in &self.conf_rev {
            data.extend(&rev.to_be_bytes());
        }
        data.extend(&self.smp_synch_asn);
        data.push(self.smp_synch);
        data.extend(&self.seq_data);
        data.extend(&self.logical_node.to_bytes());
        data
    }

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
    pub fn cal_current_phase_a ()-> [i32;1]
    {
        let now = Local::now();
        let t: f32 = now.timestamp_subsec_nanos() as f32 / 1_000_000_000.0;
        let phase_degrees: f32 = 0.0; // Example phase in degrees
        let phase_radians: f32 = phase_degrees * PI / 180.0; // Convert phase to radians
        let omega: f32 = 2.0 * PI * FREQUENCY;

        let amplitude: f32 = AMPLITUDE_CURRENT * ((omega * t + phase_radians).sin());

        //println!("t: {:?}, phase_radians: {:?}, omega: {:?}, amplitude:{:?}, amplitude i32: {:?}", t, phase_radians, omega, amplitude, amplitude as i32);
        [amplitude as i32;1]
    }

    pub fn cal_current_phase_b ()-> [i32;1]
    {
        let now = Local::now();
        let t: f32 = now.timestamp_subsec_nanos() as f32 / 1_000_000_000.0;
        let phase_degrees: f32 = 120.0; // Example phase in degrees
        let phase_radians: f32 = phase_degrees * PI / 180.0; // Convert phase to radians
        let omega: f32 = 2.0 * PI * FREQUENCY;

        let amplitude: f32 = AMPLITUDE_CURRENT * ((omega * t + phase_radians).sin());

        //println!("t: {:?}, phase_radians: {:?}, omega: {:?}, amplitude:{:?}, amplitude i32: {:?}", t, phase_radians, omega, amplitude, amplitude as i32);
        [amplitude as i32;1]
    }

    pub fn cal_current_phase_c ()-> [i32;1]
    {
        let now = Local::now();
        let t: f32 = now.timestamp_subsec_nanos() as f32 / 1_000_000_000.0;
        let phase_degrees: f32 = -120.0; // Example phase in degrees
        let phase_radians: f32 = phase_degrees * PI / 180.0; // Convert phase to radians
        let omega: f32 = 2.0 * PI * FREQUENCY;

        let amplitude: f32 = AMPLITUDE_CURRENT * ((omega * t + phase_radians).sin());

        //println!("t: {:?}, phase_radians: {:?}, omega: {:?}, amplitude:{:?}, amplitude i32: {:?}", t, phase_radians, omega, amplitude, amplitude as i32);
        [amplitude as i32;1]
    }

    pub fn cal_voltage_phase_a ()-> [i32;1]
    {
        let now = Local::now();
        let t: f32 = now.timestamp_subsec_nanos() as f32 / 1_000_000_000.0;
        let phase_degrees: f32 = 0.0; // Example phase in degrees
        let phase_radians: f32 = phase_degrees * PI / 180.0; // Convert phase to radians
        let omega: f32 = 2.0 * PI * FREQUENCY;

        let amplitude: f32 = AMPLITUDE_VOLTAGE * ((omega * t + phase_radians).sin());

        //println!("t: {:?}, phase_radians: {:?}, omega: {:?}, amplitude:{:?}, amplitude i32: {:?}", t, phase_radians, omega, amplitude, amplitude as i32);
        [amplitude as i32;1]
    }

    pub fn cal_voltage_phase_b ()-> [i32;1]
    {
        let now = Local::now();
        let t: f32 = now.timestamp_subsec_nanos() as f32 / 1_000_000_000.0;
        let phase_degrees: f32 = 120.0; // Example phase in degrees
        let phase_radians: f32 = phase_degrees * PI / 180.0; // Convert phase to radians
        let omega: f32 = 2.0 * PI * FREQUENCY;

        let amplitude: f32 = AMPLITUDE_VOLTAGE * ((omega * t + phase_radians).sin());

        //println!("t: {:?}, phase_radians: {:?}, omega: {:?}, amplitude:{:?}, amplitude i32: {:?}", t, phase_radians, omega, amplitude, amplitude as i32);
        [amplitude as i32;1]
    }

    pub fn cal_voltage_phase_c ()-> [i32;1]
    {
        let now = Local::now();
        let t: f32 = now.timestamp_subsec_nanos() as f32 / 1_000_000_000.0;
        let phase_degrees: f32 = -120.0; // Example phase in degrees
        let phase_radians: f32 = phase_degrees * PI / 180.0; // Convert phase to radians
        let omega: f32 = 2.0 * PI * FREQUENCY;

        let amplitude: f32 = AMPLITUDE_VOLTAGE * ((omega * t + phase_radians).sin());

        //println!("t: {:?}, phase_radians: {:?}, omega: {:?}, amplitude:{:?}, amplitude i32: {:?}", t, phase_radians, omega, amplitude, amplitude as i32);
        [amplitude as i32;1]
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut node = Vec::new();
        for val in &self.i_a {
            node.extend(&val.to_be_bytes());
        }
        for val in &self.q_ia {
            node.extend(&val.to_be_bytes());
        }
        for val in &self.i_b {
            node.extend(&val.to_be_bytes());
        }
        for val in &self.q_ib {
            node.extend(&val.to_be_bytes());
        }
        for val in &self.i_c {
            node.extend(&val.to_be_bytes());
        }
        for val in &self.q_ic {
            node.extend(&val.to_be_bytes());
        }
        for val in &self.i_n {
            node.extend(&val.to_be_bytes());
        }
        for val in &self.q_in {
            node.extend(&val.to_be_bytes());
        }
        for val in &self.v_a {
            node.extend(&val.to_be_bytes());
        }
        for val in &self.q_va {
            node.extend(&val.to_be_bytes());
        }
        for val in &self.v_b {
            node.extend(&val.to_be_bytes());
        }
        for val in &self.q_vb {
            node.extend(&val.to_be_bytes());
        }
        for val in &self.v_c {
            node.extend(&val.to_be_bytes());
        }
        for val in &self.q_vc {
            node.extend(&val.to_be_bytes());
        }
        for val in &self.v_n {
            node.extend(&val.to_be_bytes());
        }
        for val in &self.q_vn {
            node.extend(&val.to_be_bytes());
        }
        node
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let i_a =   [i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])];
        let q_ia =  [u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]])];
        let i_b =   [i32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]])];
        let q_ib =  [u32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]])];
        let i_c =   [i32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]])];
        let q_ic =  [u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]])];
        let i_n =   [i32::from_be_bytes([bytes[24], bytes[25], bytes[26], bytes[27]])];
        let q_in =  [u32::from_be_bytes([bytes[28], bytes[29], bytes[30], bytes[31]])];
        let v_a =   [i32::from_be_bytes([bytes[32], bytes[33], bytes[34], bytes[35]])];
        let q_va =  [u32::from_be_bytes([bytes[36], bytes[37], bytes[38], bytes[39]])];
        let v_b =   [i32::from_be_bytes([bytes[40], bytes[41], bytes[42], bytes[43]])];
        let q_vb =  [u32::from_be_bytes([bytes[44], bytes[45], bytes[46], bytes[47]])];
        let v_c =   [i32::from_be_bytes([bytes[48], bytes[49], bytes[50], bytes[51]])];
        let q_vc =  [u32::from_be_bytes([bytes[52], bytes[53], bytes[54], bytes[55]])];
        let v_n =   [i32::from_be_bytes([bytes[56], bytes[57], bytes[58], bytes[59]])];
        let q_vn =  [u32::from_be_bytes([bytes[60], bytes[61], bytes[62], bytes[63]])];

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


//Implementation about the Default function to our structs.
impl Default for EthernetFrame {
    fn default() -> Self {
        EthernetFrame {
            destination:    [0x01, 0x0c, 0xcd, 0x04, 0xff, 0xff],
            source:         [0x00, 0x1a, 0x11, 0x00, 0x00, 0x01],
            tpid:           TPID,
            tci:            TCI,
            ethertype:      ETHER_TYPE,
            payload:        SvPDU::default(),
            fcs:            [0; 4],
        }
    }
}

impl Default for SvPDU {
    fn default() -> Self {
        SvPDU {
            appid:      [0x40, 0x01],
            length:     [0x00, 0x66],
            reserved1:  [0x00, 0x00],
            reserved2:  [0x00, 0x00],
            apdu:       SmvData::default(),
        }
    }
}

impl Default for SmvData {
    fn default() -> Self {
        SmvData {
            sav_pdu_asn:    [0x60, 0x5c],
            no_asdu_asn:    [0x80, 0x01],
            no_asdu:        0x01,
            seq_asdu_asn:   [0xa2, 0x57],
            asdu_asn:       [0x30, 0x55],
            sv_id_asn:      [0x80, 0x04],
            sv_id:          [0x3430_3031],
            smp_cnt_asn:    [0x82, 0x02],
            smp_cnt:        [0x0000],
            conf_rev_asn:   [0x83, 0x04],
            conf_rev:       [0x0000_0001],
            smp_synch_asn:  [0x85, 0x01],
            smp_synch:      0x01,
            seq_data:       [0x87, 0x40],
            logical_node:   LogicalNode::default(),
        }
    }
}

impl Default for LogicalNode {
    fn default() -> Self {
        LogicalNode {
            i_a:    LogicalNode::cal_current_phase_a(),
            q_ia:   [0x0000_0000; 1],
            i_b:    LogicalNode::cal_current_phase_b(),
            q_ib:   [0x0000_0000; 1],
            i_c:    LogicalNode::cal_current_phase_c(),
            q_ic:   [0x0000_0000; 1],
            i_n:    [0; 1],
            q_in:   [0x0000_2000; 1],
            v_a:    LogicalNode::cal_voltage_phase_a(),
            q_va:   [0x0000_0000; 1],
            v_b:    LogicalNode::cal_voltage_phase_b(),
            q_vb:   [0x0000_0000; 1],
            v_c:    LogicalNode::cal_voltage_phase_c(),
            q_vc:   [0x0000_0000; 1],
            v_n:    [0; 1],
            q_vn:   [0x0000_2000; 1],
        }
    }
}


fn create_sv_packet() -> EthernetFrame {
    let destination =   [0x01, 0x0c, 0xcd, 0x04, 0xff, 0xff];
    let source =        [0x00, 0x1a, 0x11, 0x00, 0x00, 0x01];
    let tpid =            TPID;
    let tci =             TCI;
    let ethertype =       ETHER_TYPE;
    let payload =              SvPDU::default();
    let mut frame = EthernetFrame {
        destination,
        source,
        tpid,
        tci,
        ethertype,
        payload,
        fcs: [0x00, 0x00, 0x00, 0x00], // Temporary FCS
    };

    // Calculate the FCS using crc32fast
    let frame_bytes = frame.to_bytes();
    let fcs = crc32(&frame_bytes[..frame_bytes.len() - 4]).to_be_bytes();
    frame.fcs = [fcs[0], fcs[1], fcs[2], fcs[3]];

    frame
}





// The publisher function to send SV packets
async fn publisher(interface_name: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let interface_name = interface_name.trim(); // Trim any whitespace
    info!("Looking for interface: '{}'", interface_name);

    let interfaces = datalink::interfaces();

    // Print all available interfaces for debugging purposes
    println!("Available network interfaces:");
    for iface in &interfaces {
        info!("Interface: {}, MAC: {:?}", iface.name, iface.mac);
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

    let mut increment: u16 = 0; //work as a counter

    loop {
        let begin = Instant::now();
        if increment > 4799
        {
            increment = 0;
        }
        
        //Create default SV packet
        let inter = interface.clone();
        //let now = Local::now();
        //let seconds = now.second() as f32;
        let mut sv_packet = create_sv_packet();
        // Manipulate to change the values of IA,IB,IC,IN,VA,VB,VC,VN
        sv_packet.payload.apdu.smp_cnt[0] = sv_packet.payload.apdu.smp_cnt[0].wrapping_add(increment);
        if increment > 50 && increment < 100
        {
            // Implement Bad Quality to the samples
            //println!("bad quality");
            // The value of 0 is good quality
            // The value of 1 and 2 is invalid
            //The value of 3 it is questionable
            sv_packet.payload.apdu.logical_node.q_ia[0] = sv_packet.payload.apdu.logical_node.q_ia[0].wrapping_add(1);

        }

        if increment > 150 && increment < 200
        {
            // Implement Bad Quality to the samples
            //println!("bad quality");
            // The value of 0 is good quality
            // The value of 1 and 2 is invalid
            //The value of 3 it is questionable
            sv_packet.payload.apdu.logical_node.q_ia[0] = sv_packet.payload.apdu.logical_node.q_ia[0].wrapping_add(3);

        }
        // Recalculate the FCS (Frame Check Sequence)
        let frame_bytes = sv_packet.to_bytes();
        let fcs = crc32(&frame_bytes[..frame_bytes.len() - 4]).to_be_bytes();
        sv_packet.fcs = [fcs[0], fcs[1], fcs[2], fcs[3]];

        //sv_packet.payload.apdu.logical_node.i_a = cal_current_phase_a(seconds);

        // Print the SV packet for debugging
        //println!("SV Packet: {:?}", &sv_packet);

        let sv_bytes = sv_packet.to_bytes();
        let _ = tx.send_to(&sv_bytes, Some(inter))
            .expect("Failed to send packet");
        
        increment = increment.wrapping_add(1); //work as counter and add 1
        let time_reception = begin.elapsed();

        info!("Time of work of thread is: {:?}", time_reception);
        info!("Message publish");
        
        sleep(Duration::from_micros(500_000)).await;
        //sleep(Duration::from_micros(250)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    env_logger::init();
    let interface_name = env::args().nth(1).expect("Please provide an interface name as an argument");
    let publisher_task = tokio::spawn(publisher(interface_name));

    tokio::try_join!(publisher_task)?;

    Ok(())
}

