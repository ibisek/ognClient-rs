#![allow(non_snake_case)]

use std::fmt;

// use serde::{Serialize, Deserialize};
// use serde_json;
use serde_json::json;


// #[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub struct AircraftBeacon {
    pub ts: i64,
    pub prefix: String,
    pub addr: String,
    pub addr_type: AddressType,
    pub lat: f64,
    pub lon: f64,
    pub altitude: i32,
    pub agl: i32,
    pub course:u64,
    pub speed:u32,
    pub climb_rate: f64,
    pub turn_rate: f64,
    pub stealth: bool,
    pub do_not_track: bool, 
    pub aircraft_type: AircraftType,
}

impl AircraftBeacon {
    pub fn new( ts: i64, prefix: String, addr: String, addr_type: AddressType,
        lat: f64, lon: f64, altitude: i32, agl: i32,
        course:u64, speed:u32, climb_rate: f64, turn_rate: f64, 
        stealth: bool, do_not_track: bool, aircraft_type: AircraftType) -> Self {

        Self {ts, prefix, addr, addr_type, lat, lon, altitude, agl, course, speed, climb_rate, turn_rate, stealth, do_not_track, aircraft_type}
    }

    pub fn to_json_str(&self) -> String {
        // let serialized = serde_json::to_string(self).unwrap();
        let js = json!({
            "ts": self.ts,
            "prefix": self.prefix,
            "addr": self.addr,
            "addr_type": self.addr_type.value(),
            "lat": format!("{:.5}", self.lat),
            "lon": format!("{:.5}", self.lon),
            "alt": self.altitude,
            "agl": self.agl,
            "course":  self.course,
            "speed": self.speed,
            "vert_speed": format!("{:.1}", self.climb_rate),
            "turn_rate": format!("{:.1}", self.turn_rate),
            "stealth": self.stealth,
            "dnt": self.do_not_track,
            "acft_type": self.aircraft_type.value(),
        });
        
        js.to_string()
    }

    pub fn set_agl(&mut self, agl: i32) {
        self.agl = agl;
    }
}

impl fmt::Display for AircraftBeacon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#AircraftBeacon: {} | {} {} | lat:{:.4}; lon:{:.4}; alt:{:.1}m | gs:{:.1} km/h", self.ts, self.prefix, self.addr, self.lat, self.lon, self.altitude, self.speed)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AddressType {
    Unknown,
    Icao,
    Flarm,
    Ogn,
    SafeSky,
}

impl AddressType {
    pub fn from(value: u8) -> AddressType {
        match value {
            1 => AddressType::Icao,
            2 => AddressType::Flarm,
            3 => AddressType::Ogn,
            4 => AddressType::SafeSky,
            _ => AddressType::Unknown,
        }
    }

    pub fn from_short_str(value: String) -> AddressType {
        let char: &str = &value;
        match char {
            "I" => AddressType::Icao,
            "F" => AddressType::Flarm,
            "O" => AddressType::Ogn,
            "S" => AddressType::SafeSky,
            _ => AddressType::Unknown,
        }
    }

    pub fn value(&self) -> u8 {
        match *self {
            AddressType::Unknown => 0,
            AddressType::Icao => 1,
            AddressType::Flarm => 2,
            AddressType::Ogn => 3,
            AddressType::SafeSky => 4,
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            AddressType::Unknown => String::from("Unknown (0)"),
            AddressType::Icao => String::from("ICA (1)"),
            AddressType::Flarm => String::from("FLR (2)"),
            AddressType::Ogn => String::from("OGN (3)"),
            AddressType::SafeSky => String::from("SKY (4)"),
        }
    }

    pub fn as_long_str(&self) -> String {
        match *self {
            AddressType::Unknown => String::from("UNK"),
            AddressType::Icao => String::from("ICA"),
            AddressType::Flarm => String::from("FLR"),
            AddressType::Ogn => String::from("OGN"),
            AddressType::SafeSky => String::from("SKY"),
        }
    }

    pub fn as_short_str(&self) -> String {
        match *self {
            AddressType::Unknown => String::from("X"),
            AddressType::Icao => String::from("I"),
            AddressType::Flarm => String::from("F"),
            AddressType::Ogn => String::from("O"),
            AddressType::SafeSky => String::from("S"),
        }
    }
}

impl fmt::Display for AddressType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Debug, Clone, Eq, Hash)]
pub enum AircraftType {
    Undefined,
    Glider,
    TowPlane,
    Helicopter,
    Parachute,
    DropPlane,
    HangGlider,
    Paraglider,
    PistonPlane,
    JetPlane,       
    Unknown,        // 10
    Baloon,
    Airship,        // 12
    Uav,            // 13
    Reserved,
    Obstacle,       // 15
}

impl AircraftType {
    pub fn from(value: u8) -> AircraftType {
        match value {
            0 => AircraftType::Undefined,
            1 => AircraftType::Glider,
            2 => AircraftType::TowPlane,
            3 => AircraftType::Helicopter,
            4 => AircraftType::Parachute,
            5 => AircraftType::DropPlane,
            6 => AircraftType::HangGlider,
            7 => AircraftType::Paraglider,
            8 => AircraftType::PistonPlane,
            9 => AircraftType::JetPlane,
            10 => AircraftType::Unknown,
            11 => AircraftType::Baloon,
            12 => AircraftType::Airship,
            13 => AircraftType::Uav,
            14 => AircraftType::Reserved,
            15 => AircraftType::Obstacle,
            _ => AircraftType::Unknown,
        }
    }

    pub fn value(&self) -> u8 {
        match *self {
            AircraftType::Undefined => 0,
            AircraftType::Glider => 1,
            AircraftType::TowPlane => 2,
            AircraftType::Helicopter => 3,
            AircraftType::Parachute => 4,
            AircraftType::DropPlane => 5,
            AircraftType::HangGlider => 6,
            AircraftType::Paraglider => 7,
            AircraftType::PistonPlane => 8,
            AircraftType::JetPlane => 9,
            AircraftType::Unknown => 10,
            AircraftType::Baloon => 11,
            AircraftType::Airship => 12,
            AircraftType::Uav => 13,
            AircraftType::Reserved => 14,
            AircraftType::Obstacle => 15,
        }
    }
}

impl PartialEq for AircraftType {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl fmt::Display for AircraftType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

pub trait Observer<E: Clone> {
    fn notify(&mut self, event: E);
}

pub trait LineListener {
    fn notify(&mut self, line: &str);
}

pub trait AircraftBeaconListener {
    fn notify(&mut self, beacon: &AircraftBeacon);
}
