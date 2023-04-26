use chrono::prelude::*;
use lazy_static::lazy_static;
use log::{warn, error};
use regex::{Regex, Match, Captures};
use std::collections::HashSet;
use std::ops::Index;
use std::path::Prefix;
use std::str;
use std::cell::RefCell;
use std::rc::Rc;

pub mod utils;
use crate::utils::{now, from_caps, from_caps_float, from_caps_int};
mod configuration;
mod aprs_server_connection;
pub mod data_structures;

use crate::configuration::{AIRCRAFT_REGEX1, AIRCRAFT_REGEX2, AIRCRAFT_REGEX3, AIRCRAFT_REGEX4, SKY_REGEX, NEMO_REGEX, SERVER_ADDR};
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

    //rx_time: HHMMSS
    fn rx_time_to_utc_ts(rx_time: &str) -> Result<Option<i64>, std::num::ParseIntError> {

        let hour = rx_time[0..2].parse::<u32>()?;
        let min = rx_time[2..4].parse::<u32>()?;
        let sec = rx_time[4..].parse::<u32>()?;

        // TODO FIX somehow: here originate 'TS from the future' when beacon comes with local time originating from -timezones after UTC midnight
        // let utc: DateTime<Utc> = Utc::now();    
        // let utc = utc.with_hour(hour)
        //     .unwrap()
        //     .with_minute(min)
        //     .unwrap()
        //     .with_second(sec)
        //     .unwrap()
        //     .with_nanosecond(0)
        //     .unwrap();
        let mut utc: DateTime<Utc> = Utc::now();  
        match utc.with_hour(hour) {
            Some(val) => utc = val,
            None => return Ok(None),
        };
        match utc.with_minute(min) {
            Some(val) => utc = val,
            None => return Ok(None),
        };
        match utc.with_second(sec) {
            Some(val) => utc = val,
            None => return Ok(None),
        };
        utc = utc.with_nanosecond(0).unwrap();

        Ok(Some(utc.timestamp() as i64))
    }

    fn parse_signal_strength(line: &str) -> f64 {
        if !line.contains("dB") {
            return -1_f64;
        }

        let temp = &line[..line.find("dB").unwrap()];
        let temp = &temp[temp.rfind(" ").unwrap()+1..];

        let signal_strength = temp.parse::<f64>().unwrap_or(-1_f64);

        signal_strength
    }

    pub fn parse_beacon_line(&self, line: &str) -> Option<AircraftBeacon> {
        lazy_static! {
            static ref SUPPORTED_BEACONS: HashSet<String> = 
                vec!["OGN".to_string(), "FLR".to_string(), "ICA".to_string(), "SKY".to_string()]
                .into_iter().collect();
        }


        if &line.len() < &3 {
            warn!("Mangled beacon? '{}'", &line);
            return None;    // must be some mangled beacon
        }

        // println!("{} [DEBUG] line: {}", now(), line);
        let prefix = &line[0..3].to_string();

        if !SUPPORTED_BEACONS.contains(prefix) {
            if line.contains("OGNEMO") {
                return MyLineListener::parse_nemo_beacon(line);

            } else {
                // println!("Unsupported beacon: {}", line);
                return None;
            }
        }

        let beacon;
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
                // println!("[INFO] SKY rejected line: {}", line);
                return None
            }
        };

        let prefix = from_caps(&caps, 1, "").to_string();
        // let addr1 = from_caps(&caps, 2, "");
        let rx_time = from_caps(&caps, 3, "000000");
        let lat = from_caps(&caps, 4, "0");
        let lat_letter = from_caps(&caps, 5, "N");
        let lon = from_caps(&caps, 6, "0");
        let lon_letter = from_caps(&caps, 7, "E");
        // let aprs_symbol = from_caps(&caps, 8, "");
        let course: u64 = from_caps_int(&caps, 9, 0) as u64;
        let speed: u64 = from_caps_int(&caps, 10, 0) as u64; // [kt]
        let altitude: f64 = from_caps_float(&caps, 11, 0_f64); // [ft]
        let flags: u8 = u8::from_str_radix(from_caps(&caps, 12, "0"), 16).unwrap_or(0);
        let addr2 = from_caps(&caps, 13, "0").to_string();
        let vertical_speed: f64 = from_caps_float(&caps, 14, 0_f64); // [fpm]

        let ts = match Self::rx_time_to_utc_ts(rx_time) {
            Ok(val) => match val {
                Some(val) => val,
                None => return None,
            },
            Err(e) => {
                error!("Invalid rx_time '{rx_time}': {e}");
                return None;
            }
        };

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
            "".to_string(),
            Self::parse_signal_strength(&line),
        );

        Some(beacon)
    }

    /**
     * OGNEMO beacons usually contain ICA beacons.
     */
    fn parse_nemo_beacon(line: &str) -> Option<AircraftBeacon> {
        lazy_static! {
            static ref NEMO_RE: Regex = Regex::new(NEMO_REGEX).unwrap();
        }

        let caps = match NEMO_RE.captures(line) {
            Some(caps) => caps,
            None => {
                // println!("[INFO] NEMO rejected line: {}", line);
                return None
            }
        };

        let registration = from_caps(&caps, 1, "").to_string();
        let rx_time = from_caps(&caps, 2, "000000");
        let lat = from_caps(&caps, 3, "0");
        let lat_letter = from_caps(&caps, 4, "N");
        let lon = from_caps(&caps, 5, "0");
        let lon_letter = from_caps(&caps, 6, "E");
        // let aprs_symbol = from_caps(&caps, 7, "");
        let course: u64 = from_caps_int(&caps, 8, 0) as u64;
        let speed: u64 = from_caps_int(&caps, 9, 0) as u64; // [kt]
        let altitude: f64 = from_caps_float(&caps, 10, 0_f64); // [ft]
        let flags: u8 = u8::from_str_radix(from_caps(&caps, 11, "0"), 16).unwrap_or(0);
        let addr2 = from_caps(&caps, 12, "0").to_string();
        let vertical_speed: f64 = from_caps_float(&caps, 13, 0_f64); // [fpm]
        let angular_speed: f64 = from_caps_float(&caps, 14, 0_f64);

        let ts = match Self::rx_time_to_utc_ts(rx_time) {
            Ok(val) => match val {
                Some(val) => val,
                None => return None,
            },
            Err(e) => {
                error!("Invalid rx_time '{rx_time}': {e}");
                return None;
            }
        };

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

        let prefix;
        match address_type {
            AddressType::Icao => prefix = "ICA".to_string(),
            AddressType::Ogn => prefix = "OGN".to_string(),
            AddressType::Flarm => prefix = "FLR".to_string(),
            AddressType::SafeSky => prefix = "SKY".to_string(),
            _ => prefix = "NEMO".to_string()
        };

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
            angular_speed, 
            stealth,
            do_not_track,
            aircraft_type,
            registration,
            Self::parse_signal_strength(&line),
        );

        Some(beacon)
    }

    fn parse_aircraft_beacon(&self, line: &str) -> Option<AircraftBeacon> {
        // there are two very similar lines where one does not contain the 'rot' part:
        let mut regex = &self.aircraft_re4;
        // let mut regex_with_id = false;
        let mut regex_with_fpm = false;
        let mut regex_with_rot = false;
        if line.contains("rot") { 
            regex = &self.aircraft_re1;
            regex_with_rot = true;
            regex_with_fpm = true;
            // regex_with_id = true;

        } else if line.contains("fpm") {
            regex = &self.aircraft_re2;
            regex_with_fpm = true;
            // regex_with_id = true;
        } else if line.contains("id") {
            regex = &self.aircraft_re3;
            // regex_with_id = true;
         }

        let caps = match regex.captures(line) {
            Some(caps) => {
                // println!("[INFO] Accepted line: {}", line);
                caps
            } None => {
                // println!("[INFO] ALL rejected line: {}", line);
                return None
            }
        };
        // println!("CAPS: {:?}", caps);

        let prefix = from_caps(&caps, 1, "").to_string();
        let addr1 = from_caps(&caps, 2, "").to_string();
        let rx_time = from_caps(&caps, 3, "000000");
        let lat = from_caps(&caps, 4, "0");
        let lat_letter = from_caps(&caps, 5, "N");
        let lon = from_caps(&caps, 6, "0");
        let lon_letter = from_caps(&caps, 7, "E");
        // let aprs_symbol = from_caps(&caps, 8, "");
        let course: u64 = from_caps_int(&caps, 9, 0) as u64;
        let speed: u64 = from_caps_int(&caps, 10, 0) as u64; // [kt]
        let altitude: f64 = from_caps_float(&caps, 11, 0_f64); // [ft]
        let flags: u8 = u8::from_str_radix(from_caps(&caps, 12, "0"), 16).unwrap_or(0);
        // let addr2 = if regex_with_id {from_caps(&caps, 13, "").to_string()} else {"".to_string()};
        let vertical_speed: f64 = if regex_with_fpm {from_caps_float(&caps, 14, 0_f64)} else {0_f64}; // [fpm]
        let angular_speed: f64 = if regex_with_rot {from_caps_float(&caps, 15, 0_f64)} else {0_f64};
        // let flight_level: f64 = from_caps_float(&caps, 16, 0_f64);     // [flight level ~ hundrets of ft]
        // let re = Regex::new(AIRCRAFT_REGEX_ALT).unwrap();
        // let flight_level: i32 = match re.captures(line) {
        //     Some(caps) => {
        //         let fl: f64 = from_caps_float(&caps, 1, 0_f64);
        //         (fl * 100.0 * 0.3048).round() as i32  // [FL]->[m]
        //     },
        //     None => 0,
        // };

        let ts = match Self::rx_time_to_utc_ts(rx_time) {
            Ok(val) => match val {
                Some(val) => val,
                None => return None,
            },
            Err(e) => {
                error!("Invalid rx_time '{rx_time}': {e}");
                return None;
            }
        };

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
        let mut address_type: AddressType = AddressType::from(flags & 0b0000_0011);

        if address_type == AddressType::Unknown {
            match prefix.as_ref() {
                "OGN" => address_type = AddressType::Ogn,
                "ICA" => address_type = AddressType::Icao,
                "FLR" => address_type = AddressType::Flarm,
                "SKY" => address_type = AddressType::SafeSky,
                _ => (),
            }
        }

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
            "".to_string(),
            Self::parse_signal_strength(&line),
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
    do_run: bool,
    server: AprsServerConnection,
    line_listener: Rc<RefCell<MyLineListener>>,
}

impl OgnClient {
    pub fn new(username: &str) -> std::io::Result<Self> {
        // let line_listener = MyLineListener::new();
        // let line_listener = RefCell::new(MyLineListener::new());
        let line_listener = Rc::new(RefCell::new(MyLineListener::new()));

        let mut server = AprsServerConnection::new(SERVER_ADDR, username)?; 
        server.set_line_listener(Rc::clone(&line_listener));    // this finally clones the fucking reference, not the content!

        Ok(Self {
            do_run: true,
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
        while self.do_run {
            self.server.read();
        }
    }

    pub fn stop(&mut self) {
        self.do_run = false;
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
