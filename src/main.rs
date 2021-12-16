#[warn(non_snake_case)]

use std::io::{Result};
// use std::process;
// use std::String;
// use std::char;
use std::str;
use regex::Regex;
use lazy_static::lazy_static;
use chrono::prelude::*;

mod configuration;
use configuration::{AIRCRAFT_REGEX, SERVER_ADDR};

mod dataStructures;
use dataStructures::{AircraftBeacon, AddressType, AircraftType};

mod aprs_server_connection;
use aprs_server_connection::AprsServerConnection;

fn rx_time_to_utc_ts(rx_time: &str) -> u64 {
    let hour = rx_time[0..2].parse::<u32>().unwrap();
    let min = rx_time[2..4].parse::<u32>().unwrap();
    let sec = rx_time[4..].parse::<u32>().unwrap();

    let utc: DateTime<Utc> = Utc::now(); 
    let utc = utc.with_hour(hour).unwrap().with_minute(min).unwrap().with_second(sec).unwrap().with_nanosecond(0).unwrap();
    
    utc.timestamp() as u64
}

fn parse_beacon_line(line: &str) -> Option<Result<AircraftBeacon>> {
    lazy_static! {
        static ref AIRCRAFT_RE: Regex = Regex::new(AIRCRAFT_REGEX).unwrap();    
    }
    
    let caps = match AIRCRAFT_RE.captures(line) {
        Some(caps) => caps,
        None => return None,
    };
    // println!("CAPS: {:?}", caps);

    let prefix = caps.get(1).unwrap().as_str().to_string();
    // let addr1 = caps.get(2).unwrap().as_str();
    let rx_time = caps.get(3).unwrap().as_str();
    let lat = caps.get(4).unwrap().as_str();
    let lat_letter = caps.get(5).unwrap().as_str();
    let lon = caps.get(6).unwrap().as_str();
    let lon_letter = caps.get(7).unwrap().as_str();
    // let aprs_symbol = caps.get(8).unwrap().as_str();
    let course: u64 = caps.get(9).unwrap().as_str().parse().unwrap();
    let speed: u64 = caps.get(10).unwrap().as_str().parse().unwrap();           // [kt]
    let altitude: f64 = caps.get(11).unwrap().as_str().parse().unwrap();        // [ft]
    let flags: u8 = u8::from_str_radix(caps.get(12).unwrap().as_str(), 16).unwrap();
    let addr2 = caps.get(13).unwrap().as_str().to_string();
    let vertical_speed: f64 = caps.get(14).unwrap().as_str().parse().unwrap();  // [fpm]
    let angular_speed: f64 = caps.get(15).unwrap().as_str().parse().unwrap();
    // let flight_level: f64 = caps.get(16).unwrap().as_str().parse().unwrap();     // [flight level ~ hundrets of ft]

    // let re = Regex::new(AIRCRAFT_REGEX_ALT).unwrap();
    // let flight_level: i32 = match re.captures(line) {
    //     Some(caps) => {
    //         let fl: f64 = caps.get(1).unwrap().as_str().parse().unwrap();
    //         (fl * 100.0 * 0.3048).round() as i32  // [FL]->[m]
    //     },
    //     None => 0,
    // };

    let ts = rx_time_to_utc_ts(rx_time);   // convert rx_time to UTC ts
    
    // convert latitude to number:
    let signum = if lat_letter == "N" {1.0} else {-1.0};
    let lat = signum * lat[0..2].parse::<f64>().unwrap() + lat[2..].parse::<f64>().unwrap()/60.0;
    
    // convert longitude to number:
    let signum = if lon_letter == "E" {1.0} else {-1.0};
    let lon = signum * lon[0..3].parse::<f64>().unwrap() + lon[3..].parse::<f64>().unwrap()/60.0;
    
    let speed = (speed as f64 * 1.852).round() as u32;  // [kt] -> [km/h]

    // parse flags & aircraft type  STxxxxaa
    let stealth: bool = if flags & 0b1000_0000 > 0 {true} else {false};
    let do_not_track: bool = if flags & 0b0100_0000 > 0 {true} else {false};
    let aircraft_type: AircraftType = AircraftType::from(flags >> 2 & 0x0F);
    let address_type: AddressType = AddressType::from(flags & 0b0000_0011);

    let vertical_speed = vertical_speed * 0.00508;    // ft per min -> meters/s

    // convert altitude in FL to meters:
    let altitude = (altitude * 0.3048).round() as i32;
    
    let beacon = AircraftBeacon::new(ts, prefix, addr2, address_type, lat, lon, altitude, course, speed, vertical_speed, angular_speed, stealth, do_not_track, aircraft_type);
    Some(beacon)
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
    
    let mut server = AprsServerConnection::new(SERVER_ADDR, username)?;
    server.set_aprs_filter(lat, lon, range);
    server.connect();
    
    println!("Entering the loop..");
    let supported_beacons: Vec<&str> = vec!["OGN", "FLR", "ICA"];
    let mut i = 0;
    // while i <= 666 {
    loop {
        let line: String = match server.read() {
            Some(line) => line,
            None => continue,
        };

        if line.len() > 3 {
            let prefix = &line[0..3];
            if supported_beacons.contains(&prefix) {  
                i += 1;
                println!("[{:06}] {}", i, line); 
                let beacon: AircraftBeacon = match parse_beacon_line(&line) {
                    Some(res) => res.unwrap(),
                    None => continue,
                };

                println!("beacon: {}", beacon.to_json_str());

            // } else {
            //     println!("[------] {}", line); 
            }
        }
    }
    
    // Ok(())
}
