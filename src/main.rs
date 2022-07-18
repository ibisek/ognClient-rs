#[warn(non_snake_case)]

use queues::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::SystemTime;

use log::{info};
use simplelog::{ConfigBuilder, LevelFilter, SimpleLogger};

use ogn_client::data_structures::{AircraftBeacon, Observer, AddressType};
use ogn_client::OgnClient;

use ogn_client::utils::now;


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
        self.i += 1;
        // println!("beacon: #[{:06}] {}", self.i, beacon.to_json_str());
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
            let num_ogn = self.ogn_q.lock().unwrap().size();
            let num_icao = self.icao_q.lock().unwrap().size();
            let num_flarm = self.flarm_q.lock().unwrap().size();
            println!("{} [INFO] Beacon rate: {}/min, {} queued (O {} / I {} / F {})", 
                now(),
                self.i, 
                num_ogn + num_icao + num_flarm,
                num_ogn, num_icao, num_flarm
            );

            self.i = 0;
            self.time = SystemTime::now();
        }
    }
}


fn main() -> std::io::Result<()> {
    let config = ConfigBuilder::new()
        .set_target_level(LevelFilter::Info)
        .build();
    let _ = SimpleLogger::init(LevelFilter::Info, config);
    print!("\n\n## OGN CLIENT ##\n\n");


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
    
    let abl = AircraftBeaconListener::new(Arc::clone(&queue_ogn), Arc::clone(&queue_icao), Arc::clone(&queue_flarm));
    client.set_beacon_listener(abl);
    
    // client.set_beacon_listener_fn(move |beacon: AircraftBeacon| {
    //     println!("_FN: {} {}", beacon.addr_type, beacon.addr);
    // });
    
    info!("Entering the loop..");
    client.do_loop();

    info!("KOHEU.");
    Ok(())
}
