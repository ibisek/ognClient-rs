#[warn(non_snake_case)]

// use std::process;

// #[macro_use]
extern crate queues;
use queues::*;
// use queues::Queue;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::SystemTime;

mod configuration;
mod data_structures;
mod aprs_server_connection;
mod ogn_client;

use crate::data_structures::{AircraftBeacon, Observer, AddressType};
use crate::ogn_client::{OgnClient};

struct AircraftBeaconListener {
    i: u32,
    ogn_q: Arc<Mutex<Queue<AircraftBeacon>>>,
    icao_q: Arc<Mutex<Queue<AircraftBeacon>>>,
    flarm_q: Arc<Mutex<Queue<AircraftBeacon>>>,
    time: SystemTime,
}

impl AircraftBeaconListener {
    fn new(ogn_q: Arc<Mutex<Queue<AircraftBeacon>>>, 
        icao_q: Arc<Mutex<Queue<AircraftBeacon>>>, 
        flarm_q: Arc<Mutex<Queue<AircraftBeacon>>>) -> AircraftBeaconListener {
        Self {
            i:0,
            ogn_q,
            icao_q,
            flarm_q,
            time: SystemTime::now(),
        }
    }
}

impl Observer<AircraftBeacon> for AircraftBeaconListener {
    fn notify(&mut self, beacon: AircraftBeacon) {
        // println!("beacon: {}", beacon.to_json_str());
        self.i += 1;
        // println!("ABL [{:06}]: {} {} {} {:>4}m {:>3}km/h {:>8.4} {:>9.4}", self.i, beacon.ts, beacon.prefix, beacon.addr, beacon.altitude, beacon.speed, beacon.lat, beacon.lon);

        if beacon.addr_type == AddressType::Ogn {
            self.ogn_q.lock().unwrap().add(beacon).unwrap();
        } else 
        if beacon.addr_type == AddressType::Icao {
            self.icao_q.lock().unwrap().add(beacon).unwrap();
        } else 
        if beacon.addr_type == AddressType::Flarm {
            self.flarm_q.lock().unwrap().add(beacon).unwrap();
        } 

        if self.time.elapsed().unwrap().as_secs() >= 60 {
            println!("[INFO] Beacon rate: {}/min, {} queued.", 
                self.i, 
                self.ogn_q.lock().unwrap().size() + self.icao_q.lock().unwrap().size() +self.flarm_q.lock().unwrap().size(),
            );
            
            self.i = 0;
            self.time = SystemTime::now();
        }
    }
}


fn main() -> std::io::Result<()> {
    // let line = "ICA4B43D0>OGFLR,qAS,Brunnen:/202242h4654.10N/00837.09EX339/122/A=002428 !W45! id0D4B43D0 +238fpm +0.1rot 9.2dB -0.5kHz gps2x3 s7.03 h03 rDDAE09 +6.9dBm";
    // let line = "OGNC35002>OGNTRK,qAS,Sobesice:/081541h4913.50N/01634.47E'000/000/A=001053 !W81! id07C35002 +000fpm +0.0rot FL010.53 5.8dB 1e +5.2kHz gps4x6";
    // let line = "FLRDF0EFE>APRS,qAS,FYPOtest:/045949h2350.79S\01756.91E^172/098/A=004979 !W29! id22DF0EFE -197fpm +3.0rot FLRDF0EFE>APRS,qAR:/050107h2349.68S\01755.97E^330/065/A=005307 !W63! id22DF0EFE +238fpm -0.1rot 15.8dB 0e -9.4kHz gps1x3";
    // MyLineListener::new().parse_beacon_line(&line);
    // process::exit(1);

    let username = "blume";
    let lat = 49.1234;
    let lon = 16.4567;
    let range = 999999;
    
    let mut client: OgnClient = OgnClient::new(username)?;
    client.set_aprs_filter(lat, lon, range);
    client.connect();

    // let mut queue_ogn: Queue<AircraftBeacon> = queue![];
    let queue_ogn: Arc<Mutex<Queue<AircraftBeacon>>>  = Arc::new(Mutex::new(Queue::new()));
    let queue_icao: Arc<Mutex<Queue<AircraftBeacon>>>  = Arc::new(Mutex::new(Queue::new()));
    let queue_flarm: Arc<Mutex<Queue<AircraftBeacon>>>  = Arc::new(Mutex::new(Queue::new()));
    
    client.set_beacon_listener(AircraftBeaconListener::new(
        Arc::clone(&queue_ogn),
        Arc::clone(&queue_icao),
        Arc::clone(&queue_flarm),
    ));
    
    // client.set_beacon_listener_fn(move |beacon: AircraftBeacon| {
    //     println!("_FN: {} {}", beacon.addr_type, beacon.addr);
    // });
    
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

    //--

    //--

    client.do_loop();

    println!("KOHEU.");
    Ok(())
}
