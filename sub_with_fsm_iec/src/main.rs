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


use std::{env, vec};

//Synchronism Mechanism
use std::sync::{Arc, Mutex};
//Serialization crates
use serde::{Deserialize, Serialize};

// Crates that's handle time - If it is needed timezone, dates, gregorian calendar should look at chrono crate.
use std::time::Duration;
use std::time::Instant; // To measure time between a piece of the code

//crate that's create async threads
use tokio::time::{sleep, interval};
use tokio::signal;

// Crates that deal with ethernet frames
use pnet::datalink::{self, Config, NetworkInterface};
use pnet::datalink::Channel::Ethernet;
use pnet::packet::ethernet::{EtherType, EthernetPacket};
use pnet::packet::Packet;

//Crate that's generate a checksum
use crc32fast::hash as crc32;

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

const N_SAMPLES: u32 = 10;

// EthernetFrame structure definition
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

// SvPDU structure definition
#[derive(Debug, Clone)]
pub struct SvPDU {
    pub appid: [u8; 2],
    pub length: [u8; 2],
    pub reserved1: [u8; 2],
    pub reserved2: [u8; 2],
    pub apdu: SmvData,
}

// SmvData structure definition
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

// LogicalNode structure definition
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

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 18 {
            return Err("Invalid Ethernet frame length");
        }
        let destination = [bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]];
        let source = [bytes[6], bytes[7], bytes[8], bytes[9], bytes[10], bytes[11]];
        let tpid = u16::from_be_bytes([bytes[12], bytes[13]]);
        let tci = u16::from_be_bytes([bytes[14], bytes[15]]);
        let ethertype = u16::from_be_bytes([bytes[16], bytes[17]]);
        let payload = SvPDU::from_bytes(&bytes[18..bytes.len() - 4])?;
        let fcs = [bytes[bytes.len() - 4], bytes[bytes.len() - 3], bytes[bytes.len() - 2], bytes[bytes.len() - 1]];

        Ok(Self {
            destination,
            source,
            tpid,
            tci,
            ethertype,
            payload,
            fcs,
        })
    }

    pub fn verify_checksum(&self) -> bool 
        {
        let mut frame = Vec::new();
        frame.extend(&self.destination);
        frame.extend(&self.source);
        frame.extend(&self.tpid.to_be_bytes());
        frame.extend(&self.tci.to_be_bytes());
        frame.extend(&self.ethertype.to_be_bytes());
        frame.extend(&self.payload.to_bytes());
    
        let calculated_fcs = crc32(&frame);
        let provided_fcs = u32::from_be_bytes(self.fcs);
    
        calculated_fcs == provided_fcs
        }

}
    



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

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 8 {
            return Err("Invalid SvPDU length");
        }
        let appid = [bytes[0], bytes[1]];
        let length = [bytes[2], bytes[3]];
        let reserved1 = [bytes[4], bytes[5]];
        let reserved2 = [bytes[6], bytes[7]];
        let apdu = SmvData::from_bytes(&bytes[8..])?;

        Ok(Self {
            appid,
            length,
            reserved1,
            reserved2,
            apdu,
        })
    }
}

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

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 30 {
            return Err("Invalid SmvData length");
        }
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
        let logical_node = LogicalNode::from_bytes(&bytes[30..])?;

        Ok(Self {
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
        })
    }

}

impl LogicalNode {

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

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 64 {
            return Err("Invalid LogicalNode length");
        }
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

        Ok(Self {
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
        })
    }


    pub fn extract_v (&self) -> Vec<i32>{
        vec![
            self.i_a[0],
            self.i_b[0],
            self.i_c[0],
            self.i_n[0],
            self.v_a[0],
            self.v_b[0],
            self.v_c[0],
            self.v_n[0],
        ]
    }

    pub fn extract_q (&self) -> Vec<u32> {
        vec![
            self.q_ia[0],
            self.q_ib[0],
            self.q_ic[0],
            self.q_in[0],
            self.q_va[0],
            self.q_vb[0],
            self.q_vc[0],
            self.q_vn[0],

        ]
    
    }

    pub fn sum_extracted_q(&self) -> u32 {
        // Extract the values
        let values = self.extract_q();

        // Sum the values
        values.iter().sum()
        
    }

        

}

// Testing state machine aynchronous

// Define your states
#[derive(Debug)]
enum State {
    Initial,
    GetSample,
    Questionable,
    Invalid,
    Valid,
    CompleteSample,
    CheckErrorPercentage,
    ToogleMU,
    //KeepMU,
    CompleteCycle,
    }

// Define the state machine
struct FrameProcessor {
    state: State,
    cont_smp_inv: u32,
    cont_smp_valid: u32,
    error_percentage: f32,
    buffer_mu1: [[i32; N_SAMPLES as usize]; 8], // buffer MU 1 current A,B,C and N - voltage A,B,C and N
    buffer_mu2: [[i32; N_SAMPLES as usize]; 8], // buffer MU 2 current A,B,C and N - voltage A,B,C and N
    toggle_mu: bool,
}
impl FrameProcessor {
    fn new() -> Self {
        Self {
            state: State::Initial,
            cont_smp_inv: 0, // Initialize cont_invalid
            cont_smp_valid: 0,
            error_percentage: 0.0,
            buffer_mu1: [[0 ; N_SAMPLES as usize]; 8],
            buffer_mu2: [[0 ; N_SAMPLES as usize]; 8],
            toggle_mu: false,

        }
    }

    async fn process_frame(&mut self, frame: &EthernetFrame) {
        // Ensure the state machine processes at each tick
        let mut ticker = interval(Duration::from_micros(1));
        info!("frame inside FSM {:?}", frame);
        
        loop {
            // Wait for the tick, so the state machine each ticker
            ticker.tick().await;
            info!("System ticked");
            

            
            // Here we need to apply our logic to the states the transitions will be handled in the transition function

            match self.state {
                State::Initial => 
                {
                    info!("State: Initial");
                }
                
                State::GetSample =>
                { 
                    info!("State: Get Sample");
                    self.valid_qual_smp(frame).await;   
                }
                
                State::Questionable => 
                {
                    info!("State: Questionable");
                }
                
                State::Invalid => 
                {
                    info!("State: Invalid");
                    self.valid_cont_samp_inv().await;
                }

                State::Valid => 
                {
                    info!("State: Valid");
                    self.valid_cont_samp_valid(frame).await;
                }

                State::CompleteSample => 
                {
                    info!("State: CompleteSample");
                    break;
                }

                State::CheckErrorPercentage =>
                { 
                    info!("State: CheckErrorPercentage");
                    self.valid_error_percentage(frame).await;    
                }
                //State::KeepMU => self.parse_frame(frame).await,
                
                State::ToogleMU =>
                { 
                    info!("State: ToogleMU");
                    self.toogle_sv().await;
                }
                
                State::CompleteCycle => 
                {
                    info!("State: CompleteCycle");
                    self.reset_variables().await;
                    break;
                }
            }

            // Automatically transition to the next state on each tick
            self.transition().await;

            
        }
        self.state = State::GetSample; // Reset state for the next frame
    }

    async fn transition(&mut self) {
        match self.state {
            State::Initial => self.state = State::GetSample,

            State::GetSample =>{}

            State::Invalid =>{}

            State::Valid =>{}

            State::Questionable => self.state = State::CompleteSample,

            State::CompleteSample => self.state = State::GetSample,

            State::CheckErrorPercentage =>{}

            State::ToogleMU => self.state = State::CompleteCycle,

            State::CompleteCycle => self.state = State::GetSample,
        }
    }

    async fn valid_qual_smp(&mut self, _frame: &EthernetFrame) {
        // Validation logic here
        // On success:
        //self.state = State::Validating;
        info!("Verify Quality of the Sample");
        let qual_message = _frame.payload.apdu.logical_node.sum_extracted_q();
        info!("Logical Node : {:?}", qual_message);
        if qual_message == 0x00
        {
            self.state = State::Valid;
            info!("Get Sample -> Valid");
        }

        else if qual_message > 0x00 || qual_message < 0x03 
        {
            self.state = State::Invalid;
            info!("Get Sample -> Invalid");
        }

        else if qual_message >= 0x03
        {
            self.state = State::Questionable;
            info!("Get Sample -> Questionable");

        }
        else 
        {
            //self.state = State::Error;
            info!("Get Sample -> Error");    
        } 

        // On failure:
        // self.state = State::Error;
    }

    async fn valid_cont_samp_inv(&mut self) {
        info!("Validate the value of cont samples invalid: {:?}", self.cont_smp_inv);
        self.cont_smp_inv += 1;
   
        if  self.cont_smp_inv < N_SAMPLES
        {
            self.state = State::CompleteSample;
            info!("Invalid -> Complete Sample");
        }
        else if self.cont_smp_inv >= N_SAMPLES 
        {
            self.state = State::ToogleMU;
            info!("Invalid -> Toogle MU");
        }
        else 
        {
            //self.state = State::Error;
            info!("Invalid -> Error");    
        }      
        
    }

    async fn valid_cont_samp_valid(&mut self, _frame: &EthernetFrame) {

    }

    async fn toogle_sv(&mut self){
        self.toggle_mu = !self.toogle_mu;

    }

    async fn valid_error_percentage(&mut self, _frame: &EthernetFrame){

    }

    async fn reset_variables(&mut self) {
        // Validation logic here
        // On success:
        //self.state = State::Validating;
        info!("Reset");
        // On failure:
        // self.state = State::Error;
    }

}




#[tokio::main]
async fn main() {
    // Initialize the logger
    env_logger::init();

    // Retrieve the network interface to use
    let interface_name = env::args().nth(1).unwrap_or_else(|| "eth0".to_string());
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .filter(|iface| iface.name == interface_name)
        .next()
        .expect("Could not find the specified network interface");

    // Create a channel to receive Ethernet frames
    let mut config = Config::default();
    config.read_timeout = Some(Duration::from_millis(1000));

    let (mut _tx, mut rx) = match datalink::channel(&interface, config) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e),
    };

    info!("Listening on interface {}", interface.name);

    let mut frame_processor = FrameProcessor::new();

    tokio::select! {
        _ = async {
            loop {
                let begin = Instant::now();
                match rx.next() {
                    Ok(frame) => {
                        let packet = EthernetPacket::new(frame).unwrap();
                        if packet.get_ethertype() == EtherType(TPID) || packet.get_ethertype() == EtherType(ETHER_TYPE) {
                            match EthernetFrame::from_bytes(packet.packet()) {
                                Ok(ethernet_frame) =>{ 
                                    info!("Received Ethernet Frame: {:?}", ethernet_frame);
                                    let validate_frame = EthernetFrame::verify_checksum(&ethernet_frame);
                                    if validate_frame {
                                    frame_processor.process_frame(&ethernet_frame).await;
                                    }
                                },
                                Err(e) => warn!("Failed to parse Ethernet frame: {:?}", e),
                            }
                            
                            
                        }                     
                    }
                    
                    Err(e) => error!("An error occurred while reading: {}", e),
                }
                let time_reception = begin.elapsed();

                info!("Time of work of thread is: {:?}", time_reception);
                sleep(Duration::from_micros(400)).await;
            }
        } => {},
        _ = signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down.");
        },
    }
}