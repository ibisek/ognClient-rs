use chrono::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;
use std::io::Result;
use std::str;
use std::cell::RefCell;
use std::rc::Rc;

pub mod utils;
use crate::utils::now;
mod configuration;
mod aprs_server_connection;
pub mod data_structures;

use crate::configuration::{AIRCRAFT_REGEX1, AIRCRAFT_REGEX2, AIRCRAFT_REGEX3, AIRCRAFT_REGEX4, SKY_REGEX, SERVER_ADDR};
use self::aprs_server_connection::AprsServerConnection;
use self::data_structures::{AddressType, AircraftBeacon, AircraftType, Observer};


//#[derive(Clone)]
pub struct MyLineListener {
    beacon_listener: Option<Rc<RefCell<dyn Observer<AircraftBeacon>>>>,
    beacon_listener_fn: Option<Box<dyn Fn(AircraftBeacon)>>,
    aircraft_re1: Regex,
    aircraft_re2: Regex,
    aircraft_re3: Regex,
    aircraft_re4: Regex,
}

impl MyLineListener {
    pub fn new() -> MyLineListener {
        MyLineListener {
            beacon_listener: None,
            beacon_listener_fn: None,
            aircraft_re1: Regex::new(AIRCRAFT_REGEX1).unwrap(),
            aircraft_re2: Regex::new(AIRCRAFT_REGEX2).unwrap(),
            aircraft_re3: Regex::new(AIRCRAFT_REGEX3).unwrap(),
            aircraft_re4: Regex::new(AIRCRAFT_REGEX4).unwrap(),
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

    pub fn parse_beacon_line(&self, line: &str) -> Option<AircraftBeacon> {
        lazy_static! {
            static ref SUPPORTED_BEACONS: Vec<String> =
                vec!["OGN".to_string(), "FLR".to_string(), "ICA".to_string(), "SKY".to_string()];
        }

        // println!("{} [DEBUG] line: {}", now(), line);
        let prefix = &line[0..3].to_string();
        if !SUPPORTED_BEACONS.contains(&prefix) {
            // println!("Unsupported beacon: {}", line);
            return None;
        }

        let mut beacon = None;
        if prefix == "SKY" {
            beacon = MyLineListener::parse_sky_beacon(line);
        } else {
            beacon = self.parse_aircraft_beacon(line);
        }

        beacon
    }

    fn parse_sky_beacon(line: &str) -> Option<AircraftBeacon> {
        lazy_static! {
            static ref SKY_RE: Regex = Regex::new(SKY_REGEX).unwrap();
        }

        let caps = match SKY_RE.captures(line) {
            Some(caps) => caps,
            None => {
                println!("[INFO] Rejected line: {}", line);
                return None
            }
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

        let ts = Self::rx_time_to_utc_ts(rx_time); // convert rx_time to UTC ts
        // convert latitude to number:
        let signum = if lat_letter == "N" { 1.0 } else { -1.0 };
        let pos = lat.find('.').unwrap();   // 5140.77 -> 51 40.77
        let lat = signum * lat[0..(pos-2)].parse::<f64>().unwrap() + lat[(pos-2)..].parse::<f64>().unwrap() / 60.0;
        // convert longitude to number:
        let signum = if lon_letter == "E" { 1.0 } else { -1.0 };
        let pos = lon.find('.').unwrap();   // 12345.67 -> 123 45.67
        let lon = signum * lon[0..(pos-2)].parse::<f64>().unwrap() + lon[(pos-2)..].parse::<f64>().unwrap() / 60.0;

        let speed = (speed as f64 * 1.852).round() as u32; // [kt] -> [km/h]
        // parse flags & aircraft type  STxxxxaa
        let stealth: bool = if flags & 0b1000_0000 > 0 { true } else { false };
        let do_not_track: bool = if flags & 0b0100_0000 > 0 { true } else { false };
        let aircraft_type: AircraftType = AircraftType::from(flags >> 2 & 0x0F);
        let address_type: AddressType = AddressType::SafeSky;

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
            0,
            course,
            speed,
            vertical_speed,
            0.0,  // not known from the line 
            stealth,
            do_not_track,
            aircraft_type,
        );

        Some(beacon)
    }

    fn parse_aircraft_beacon(&self, line: &str) -> Option<AircraftBeacon> {
        // there are two very similar lines where one does not contain the 'rot' part:
        let mut regex = &self.aircraft_re4;
        let mut regex_with_id = false;
        let mut regex_with_fpm = false;
        let mut regex_with_rot = false;
        if line.contains("rot") { 
            regex = &self.aircraft_re1;
            regex_with_rot = true;
            regex_with_fpm = true;
            regex_with_id = true;

        } else if line.contains("fpm") {
            regex = &self.aircraft_re2;
            regex_with_fpm = true;
            regex_with_id = true;
        } else if line.contains("id") {
            regex = &self.aircraft_re3;
            regex_with_id = true;
         }

        let caps = match regex.captures(line) {
            Some(caps) => {
                // println!("[INFO] Accepted line: {}", line);
                caps
            } None => {
                // println!("[INFO] Rejected line: {}", line);
                return None
            }
        };
        // println!("CAPS: {:?}", caps);

        let prefix = caps.get(1).unwrap().as_str().to_string();
        let addr1 = caps.get(2).unwrap().as_str().to_string();
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
        let addr2 = if regex_with_id {caps.get(13).unwrap().as_str().to_string()} else {"".to_string()};
        let vertical_speed: f64 = if regex_with_fpm {caps.get(14).unwrap().as_str().parse().unwrap()} else {0_f64}; // [fpm]
        let angular_speed: f64 = if regex_with_rot {caps.get(15).unwrap().as_str().parse().unwrap()} else {0_f64};
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
        let pos = lat.find('.').unwrap();   // 5140.77 -> 51 40.77
        let lat = signum * lat[0..(pos-2)].parse::<f64>().unwrap() + lat[(pos-2)..].parse::<f64>().unwrap() / 60.0;
        // convert longitude to number:
        let signum = if lon_letter == "E" { 1.0 } else { -1.0 };
        let pos = lon.find('.').unwrap();   // 12345.67 -> 123 45.67
        let lon = signum * lon[0..(pos-2)].parse::<f64>().unwrap() + lon[(pos-2)..].parse::<f64>().unwrap() / 60.0;

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
            addr1,
            address_type,
            lat,
            lon,
            altitude,
            0,
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

    pub fn set_beacon_listener(&mut self, listener: impl Observer<AircraftBeacon> + 'static) {
        self.beacon_listener = Some(Rc::new(RefCell::new(listener)));
    }

    pub fn set_beacon_listener_fn<F: 'static>(&mut self, callback: F) 
    where
        F: Fn(AircraftBeacon)
    {
        self.beacon_listener_fn = Some(Box::new(callback));
    }

}

impl Observer<String> for MyLineListener {
    fn notify(&mut self, line: String) {
        // println!("MLL.line: {}", line);
        let beacon_opt = self.parse_beacon_line(&line);
        
        if beacon_opt.is_some() {
            let beacon = beacon_opt.unwrap();

            if self.beacon_listener.is_some() {
                self.beacon_listener.as_mut().unwrap().borrow_mut().notify(beacon.clone());
            }

            if self.beacon_listener_fn.is_some() {
                (self.beacon_listener_fn.as_mut().unwrap())(beacon);
            }
        }
    }
}

pub struct OgnClient {
    server: AprsServerConnection,
    line_listener: Rc<RefCell<MyLineListener>>,
}

impl OgnClient {
    pub fn new(username: &str) -> Result<Self> {
        // let line_listener = MyLineListener::new();
        // let line_listener = RefCell::new(MyLineListener::new());
        let line_listener = Rc::new(RefCell::new(MyLineListener::new()));

        let mut server = AprsServerConnection::new(SERVER_ADDR, username)?; 
        server.set_line_listener(Rc::clone(&line_listener));    // this finally clones the fucking reference, not the content!

        Ok(Self {
            server: server,
            line_listener: line_listener,
        })
    }

    /// Needs to be set before connect()!
    pub fn set_aprs_filter(&mut self, lat: f64, lon: f64, range: u32) {
        self.server.set_aprs_filter(lat, lon, range);
    }

    pub fn connect(& mut self) {
        self.server.connect();
        
    }

    pub fn do_loop(&mut self) {
        loop {
            self.server.read();
        }
    }

    pub fn set_beacon_listener(&mut self, listener: impl Observer<AircraftBeacon> + 'static) {
        // self.line_listener.as_mut().unwrap().borrow_mut().set_beacon_listener(listener);
        self.line_listener.borrow_mut().set_beacon_listener(listener);
    }

    pub fn set_beacon_listener_fn<F: 'static>(&mut self, callback: F) 
    where
        F: Fn(AircraftBeacon)
    {
        self.line_listener.borrow_mut().set_beacon_listener_fn(callback);
    }
    
}
