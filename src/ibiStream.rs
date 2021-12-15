use std::{thread, time};
use std::io::prelude::*;
use std::io::{Write, BufReader, LineWriter, Result};
use std::net::TcpStream;

use crate::configuration::{DELAY_MS, HANDSHAKE};

pub struct IbiStream {
    address: String,
    reader: Option<BufReader<TcpStream>>,
    writer: Option<LineWriter<TcpStream>>,
    nextReconnectDelay: u64, // [s]
}

impl IbiStream {

    pub fn new(address: &str) -> Result<Self> {
        Ok(Self {address: String::from(address), reader: None, writer: None, nextReconnectDelay:1 })
    }

    pub fn connect(&mut self) {
	print!("Connecting.. ");
        let stream = match TcpStream::connect(self.address.clone()) {
            Ok(stream) => {
                println!("ok");
                self.nextReconnectDelay = 1;    // [s]
                stream
            }
            Err(e) => {
                println!("again in {} s", self.nextReconnectDelay);
                thread::sleep(time::Duration::from_millis(self.nextReconnectDelay*1000));
                self.nextReconnectDelay *= 2;
                return
            }
        };

        // stream.set_nonblocking(true).expect("set_nonblocking call failed");

        // both BufReader and LineWriter need to own a stream. This can be done by cloning the stream to simulate splitting Tx & Rx with try_clone()
        self.writer = Some(LineWriter::new(stream.try_clone().unwrap()));
        self.reader = Some(BufReader::new(stream));
	
        thread::sleep(time::Duration::from_millis(DELAY_MS));   // give the server some time to respond
        self.write(&HANDSHAKE);
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

