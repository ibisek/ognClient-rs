use chrono::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;
use std::io::Result;
use std::str;

use crate::aprs_server_connection::AprsServerConnection;
use crate::configuration::{AIRCRAFT_REGEX, SERVER_ADDR};
use crate::data_structures::{AddressType, AircraftBeacon, AircraftType, Observer};

pub struct MyLineListener {
    i: u32,
}

impl MyLineListener {
    fn new() -> MyLineListener {
        MyLineListener {
            i: 0,
        }
    }
    fn rx_time_to_utc_ts(rx_time: &str) -> u64 {
        let hour = rx_time[0..2].parse::<u32>().unwrap();
        let min = rx_time[2..4].parse::<u32>().unwrap();
        let sec = rx_time[4..].parse::<u32>().unwrap();

        let utc: DateTime<Utc> = Utc::now();
        let utc = utc
            .with_hour(hour)
            .unwrap()
            .with_minute(min)
            .unwrap()
            .with_second(sec)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();
        utc.timestamp() as u64
    }

    fn parse_beacon_line(&self, line: &str) -> Option<AircraftBeacon> {
        lazy_static! {
            static ref AIRCRAFT_RE: Regex = Regex::new(AIRCRAFT_REGEX).unwrap();
            static ref SUPPORTED_BEACONS: Vec<String> =
                vec!["OGN".to_string(), "FLR".to_string(), "ICA".to_string()];
        }
        let prefix = &line[0..3].to_string();
        if !SUPPORTED_BEACONS.contains(&prefix) {
            return None;
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
        let speed: u64 = caps.get(10).unwrap().as_str().parse().unwrap(); // [kt]
        let altitude: f64 = caps.get(11).unwrap().as_str().parse().unwrap(); // [ft]
        let flags: u8 = u8::from_str_radix(caps.get(12).unwrap().as_str(), 16).unwrap();
        let addr2 = caps.get(13).unwrap().as_str().to_string();
        let vertical_speed: f64 = caps.get(14).unwrap().as_str().parse().unwrap(); // [fpm]
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

        let ts = Self::rx_time_to_utc_ts(rx_time); // convert rx_time to UTC ts
        // convert latitude to number:
        let signum = if lat_letter == "N" { 1.0 } else { -1.0 };
        let lat =
            signum * lat[0..2].parse::<f64>().unwrap() + lat[2..].parse::<f64>().unwrap() / 60.0;
        // convert longitude to number:
        let signum = if lon_letter == "E" { 1.0 } else { -1.0 };
        let lon =
            signum * lon[0..3].parse::<f64>().unwrap() + lon[3..].parse::<f64>().unwrap() / 60.0;

        let speed = (speed as f64 * 1.852).round() as u32; // [kt] -> [km/h]
        // parse flags & aircraft type  STxxxxaa
        let stealth: bool = if flags & 0b1000_0000 > 0 { true } else { false };
        let do_not_track: bool = if flags & 0b0100_0000 > 0 { true } else { false };
        let aircraft_type: AircraftType = AircraftType::from(flags >> 2 & 0x0F);
        let address_type: AddressType = AddressType::from(flags & 0b0000_0011);

        let vertical_speed = vertical_speed * 0.00508; // ft per min -> meters/s
        // convert altitude in FL to meters:
        let altitude = (altitude * 0.3048).round() as i32;

        let beacon = AircraftBeacon::new(
            ts,
            prefix,
            addr2,
            address_type,
            lat,
            lon,
            altitude,
            course,
            speed,
            vertical_speed,
            angular_speed,
            stealth,
            do_not_track,
            aircraft_type,
        );
        Some(beacon)
    }
}

impl Observer<String> for MyLineListener {
    fn notify(&mut self, line: &String) {
        // println!("MLL {}", line);
        let beacon_opt = self.parse_beacon_line(&line);
        
        if beacon_opt.is_some() {
            let beacon = beacon_opt.unwrap();

            self.i += 1;
            println!("MLL [{:06}]: {} {} {} {:>4}m {:>3}km/h", self.i, beacon.ts, beacon.prefix, beacon.addr, beacon.altitude, beacon.speed);

            // for listener in &self.beacon_listeners.iter_mut() {
            //     listener.notify(&beacon);
            // }
        }
    }
}

pub struct OgnClient {
    server: AprsServerConnection,
    beacon_listeners: Vec<Box<dyn Observer<AircraftBeacon>>>,
    // line_listener: Option<dyn Observer<String>>,
    line_listener: Option<MyLineListener>,
}

// https://stackoverflow.com/questions/44928882/why-do-i-get-the-error-the-trait-foo-is-not-implemented-for-mut-t-even-th
impl Observer<String> for &mut OgnClient {
    fn notify(&mut self, line: &String) {
        println!("XX lyne: {}", line);
        // let beacon_opt = self.parse_beacon_line(&line);
        // if beacon_opt.is_some() {
        //     let beacon = beacon_opt.unwrap();
        //     for listener in &self.beacon_listeners {
        //         listener.notify(&beacon);
        //     }
        // }
    }
}

impl OgnClient {
    pub fn new(username: &str) -> Result<Self> {
        Ok(Self {
            server: AprsServerConnection::new(SERVER_ADDR, username)?,
            beacon_listeners: Vec::new(),
            line_listener: None,
        })
    }

    /// needs to be set before connect()
    pub fn set_aprs_filter(&mut self, lat: f64, lon: f64, range: u32) {
        self.server.set_aprs_filter(lat, lon, range);
    }

    pub fn connect(&mut self) {
        self.server.connect();
        // self.server.add_line_listener(Box::new(self));
        self.server.add_line_listener(Box::new(MyLineListener::new()));
    }

    pub fn do_loop(&mut self) {
        loop {
            self.server.read();
        }
    }

    pub fn add_beacon_listener(&mut self, observer: Box<dyn Observer<AircraftBeacon>>) {
        // TODO check already in there
        self.beacon_listeners.push(observer);
    }
    
}

// #[derive(Debug, Clone)]
// pub struct Event {
//     line: String,
//     beacon: Option<AircraftBeacon>,
// }

// pub trait Observable {
//     fn register(&mut self, observer: Box<dyn Observer<Event>>);
// }

// impl Observable for OgnClient {
//     fn register(&mut self, observer: Box<dyn Observer<Event>>) {
//         self.listeners.push(observer);
//     }
// }

// pub struct EventConsumer {}

// impl Observer<Event> for EventConsumer {
//     fn notify(&self, event: &Event) {
//         println!("GOT EVENT\n beaconek: {:?}", event.line);
//         if event.beacon.is_some() {
//             println!(" beacon: {:?}", event.beacon.as_ref().unwrap());
//         } else {
//             println!(" No beacon set");
//         }
//     }
// }
