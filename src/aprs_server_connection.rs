use std::{thread, time};
use std::io::prelude::*;
use std::io::{Write, BufReader, LineWriter, Result};
use std::net::TcpStream;

use crate::configuration::{DELAY_MS, DEFAULT_APRS_FILTER};

pub struct AprsServerConnection {
    address: String,
    reader: Option<BufReader<TcpStream>>,
    writer: Option<LineWriter<TcpStream>>,
    next_reconnect_delay: u64, // [s]
    aprs_filter: String,
    username: String,
}

impl AprsServerConnection {

    pub fn new(address: &str, username: &str) -> Result<Self> {
        Ok(Self {address: String::from(address), 
            reader: None, 
            writer: None, 
            next_reconnect_delay:1, 
            aprs_filter: DEFAULT_APRS_FILTER.to_string(),
            username: String::from(username) })
    }

    pub fn connect(&mut self) {
	print!("Connecting.. ");
        let stream = match TcpStream::connect(self.address.clone()) {
            Ok(stream) => {
                println!("ok");
                self.next_reconnect_delay = 1;    // [s]
                stream
            }
            Err(_) => {
                println!("again in {} s", self.next_reconnect_delay);
                thread::sleep(time::Duration::from_millis(self.next_reconnect_delay*1000));
                self.next_reconnect_delay *= 2;
                return
            }
        };

        // stream.set_nonblocking(true).expect("set_nonblocking call failed");

        thread::sleep(time::Duration::from_millis(DELAY_MS));   // give the server some time to respond

        // both BufReader and LineWriter need to own a stream. This can be done by cloning the stream to simulate splitting Tx & Rx with try_clone()
        self.writer = Some(LineWriter::new(stream.try_clone().unwrap()));
        self.reader = Some(BufReader::new(stream));

        let handshake = format!("user {} pass -1 vers rustClient 0.1 filter {}", self.username, self.aprs_filter);
        println!("XX HANDSH: {}", handshake);
        self.write(&handshake).unwrap();
    }

    pub fn set_aprs_filter(&mut self, lat: f64, lon: f64, range: u32) {
        self.aprs_filter = format!("r/{:.4}/{:.4}/{}", lat, lon, range);
    }

    pub fn write(&mut self, message: &str) -> Result<()> {
        self.writer.as_mut().unwrap().write(&message.as_bytes())?;
        self.writer.as_mut().unwrap().write(&['\n' as u8])?;  // This will also signal a `writer.flush()`
        Ok(())
    }

    pub fn read(&mut self) -> Option<String> {
        let mut line = String::new();

        let num_read = match self.reader.as_mut().unwrap().read_line(&mut line) {
            Ok(val) => val,
            Err(_) => 0,
        };

        if num_read == 0 {
            self.connect();
            return None;
        }

        let line = String::from(line.trim()); // Remove the trailing "\n"
        Some(line)
    }
}
