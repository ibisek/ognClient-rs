
use std::{thread, time, time::Duration};
use std::io::prelude::*;
use std::io::{Write, BufReader, LineWriter, Result};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::SystemTime;
use std::io::ErrorKind::ConnectionReset;

use crate::configuration::{DELAY_MS, DEFAULT_APRS_FILTER};
use crate::data_structures::Observer;


pub struct AprsServerConnection {
    address: String,
    reader: Option<BufReader<TcpStream>>,
    writer: Option<LineWriter<TcpStream>>,
    next_reconnect_delay: u64, // [s]
    aprs_filter: String,
    username: String,
    // line_listeners: Vec<Box<dyn Observer<String>>>,
    // pub line_listener: Option<Box<dyn Observer<String>>>,
    pub line_listener: Option<Rc<RefCell<dyn Observer<String>>>>,
    // pub line_listener_fn: Option<Box<dyn Fn(String)>>,
    last_keepalive_ts: SystemTime,
}

impl AprsServerConnection {

    pub fn new(address: &str, username: &str) -> Result<Self> {
        Ok(Self {address: String::from(address), 
            reader: None, 
            writer: None, 
            next_reconnect_delay:1, 
            aprs_filter: DEFAULT_APRS_FILTER.to_string(),
            username: String::from(username),
            // line_listeners: Vec::new(),
            line_listener: None,
            // line_listener_fn: None,
            last_keepalive_ts: SystemTime::now(),
        })
    }

    pub fn connect(&mut self) {
        print!("Connecting.. ");
        let stream = match TcpStream::connect(self.address.clone()) {
            Ok(stream) => {
                println!("ok");
                // stream.set_nonblocking(true).expect("[ERROR] set_nonblocking call failed");
                stream.set_read_timeout(Some(Duration::new(10, 0))).expect("[ERROR] set_read_timeout call failed");
                self.next_reconnect_delay = 1;    // [s]
                stream
            }
            Err(_) => {
                println!("again in {}s", self.next_reconnect_delay);
                thread::sleep(time::Duration::from_millis(self.next_reconnect_delay*1000));
                self.next_reconnect_delay *= 2;
                return
            }
        };

        thread::sleep(time::Duration::from_millis(DELAY_MS));   // give the server some time to respond

        // both BufReader and LineWriter need to own a stream. This can be done by cloning the stream to simulate splitting Tx & Rx with try_clone()
        self.writer = Some(LineWriter::new(stream.try_clone().unwrap()));
        self.reader = Some(BufReader::new(stream));

        let handshake = format!("user {} pass -1 vers rustClient 0.0.1 filter {}", self.username, self.aprs_filter);
        self.write(&handshake).unwrap();
    }

    /// Sets APRS filter to receive beacons from the desired area only. Use before calling the connect().
    /// @param lat latitude of the area center [deg]
    /// @param lon longitude of the area center [deg]
    /// @param range [km]
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

        let num_read = match self.reader.as_mut() {
            Some(reader) => {
                match reader.read_line(&mut line) {
                    Ok(val) => val,
                    Err(e) => { // e.g. 'stream did not contain valid UTF-8' / 'Connection reset by peer (os error 104)'
                        println!("[ERROR] when reading from stream: '{:?}' - {}", e.kind(), e);
                        // @see https://doc.rust-lang.org/stable/std/io/enum.ErrorKind.html
                        // kind: InvalidData / 
                        // if e.kind() == ConnectionReset {
                        //     println!("Reconnect?");
                        // }
                        0
                    },
                }
            },
            None => 0,
        };

        // if num_read == 0 {
        //     self.connect();
        //     return None;
        // }

        let line = String::from(line.trim()); // Remove the trailing "\n"
        if line.len() > 0 {
            // self.notify_line_listeners(line.clone());
            self.notify_line_listener(line.clone());
        }

        // self.send_keepalive_msg();

        Some(line)
    }

    // fn notify_line_listeners(&mut self, line: String) {
    //     for listener in self.line_listeners.iter_mut() {
    //         listener.notify(line);
    //     }
    // }   

    fn notify_line_listener(&mut self, line: String) {
        if self.line_listener.is_some() {
            self.line_listener.as_mut().unwrap().borrow_mut().notify(line);
        }

        // if self.line_listener_fn.is_some() {
        //     (self.line_listener_fn.as_mut().unwrap())(line);
        // }
    }   

    /// Sends a generic comment/mesage into the socket stream to keep the connection alive.
    fn send_keepalive_msg(&mut self) {
        if self.last_keepalive_ts.elapsed().unwrap().as_secs() >= 2*60 {  // 2 min interval
            self.write(&"#keepalive").unwrap();
            self.last_keepalive_ts = SystemTime::now();
        }
    }

    // pub fn add_line_listener(&mut self, listener: Box<dyn Observer<String>>) {
    // // pub fn add_line_listener(&mut self, listener: &'static impl Observer<String>) {
    // // pub fn add_line_listener(&mut self, listener: &(impl Observer<String> + 'static)) {
    //     // TODO check already present
    //     if !self.line_listeners.contains(listener) {
    //         self.line_listeners.push(listener);
    //     }
    // }

    // pub fn set_line_listener(&mut self, listener: impl Observer<String> + 'static) {
    pub fn set_line_listener(&mut self, listener: Rc<RefCell<impl Observer<String> + 'static>>) {
        self.line_listener = Some(listener);
    }

    // pub fn set_line_listener_fn<F: 'static>(&mut self, handler: F)
    // where
    //     F: Fn(String) -> ()
    // {
    //     self.line_listener_fn = Some(Box::new(handler));
    // }
}
