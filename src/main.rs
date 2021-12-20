#[warn(non_snake_case)]

use std::io::{Result};
use std::process;
// use std::String;
// use std::char;
use std::str;
use regex::Regex;
use lazy_static::lazy_static;
use chrono::prelude::*;

mod configuration;
mod data_structures;
mod aprs_server_connection;
mod ogn_client;

use crate::configuration::{AIRCRAFT_REGEX, SERVER_ADDR};
use crate::data_structures::{AircraftBeacon, AddressType, AircraftType, Observer};
use crate::aprs_server_connection::AprsServerConnection;
use crate::ogn_client::{OgnClient};


struct AircraftBeaconListener {}
impl Observer<AircraftBeacon> for AircraftBeaconListener {
    fn notify(&self, beacon: &AircraftBeacon) {
        println!("beacon: {}", beacon.to_json_str());
    }
}

fn main() -> std::io::Result<()> {
    // let line = "ICA4B43D0>OGFLR,qAS,Brunnen:/202242h4654.10N/00837.09EX339/122/A=002428 !W45! id0D4B43D0 +238fpm +0.1rot 9.2dB -0.5kHz gps2x3 s7.03 h03 rDDAE09 +6.9dBm";
    // let line = "OGNC35002>OGNTRK,qAS,Sobesice:/081541h4913.50N/01634.47E'000/000/A=001053 !W81! id07C35002 +000fpm +0.0rot FL010.53 5.8dB 1e +5.2kHz gps4x6";
    // parse_beacon_line(&line);
    // process::exit(1);

    let username = "blume";
    let lat = 49.1234;
    let lon = 16.4567;
    let range = 999999;
    
    let mut client: OgnClient = OgnClient::new(username)?;
    client.set_aprs_filter(lat, lon, range);
    client.connect();
    
    println!("Entering the loop..");
    // let supported_beacons: Vec<&str> = vec!["OGN", "FLR", "ICA"];
    // let mut i = 0;
    // // while i <= 666 {
    // loop {
    //     let line: String = match server.read() {
    //         Some(line) => line,
    //         None => continue,
    //     };

    //     if line.len() > 3 {
    //         let prefix = &line[0..3];
    //         if supported_beacons.contains(&prefix) {  
    //             i += 1;
    //             println!("[{:06}] {}", i, line); 
    //             let beacon: AircraftBeacon = match parse_beacon_line(&line) {
    //                 Some(res) => res.unwrap(),
    //                 None => continue,
    //             };

    //             println!("beacon: {}", beacon.to_json_str());

    //         // } else {
    //         //     println!("[------] {}", line); 
    //         }
    //     }
    // }

    client.do_loop();

    println!("KOHEU.");
    Ok(())
}
