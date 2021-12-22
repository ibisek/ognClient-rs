pub const DELAY_MS: u64 = 1*1000;

// const SERVER_ADDR: &str = "localhost:8888";
pub const SERVER_ADDR: &str = "aprs.glidernet.org:14580";   // port filtered
// pub const SERVER_ADDR: &str = "aprs.glidernet.org:10152";   // port unfiltered .. whatever that means

pub const DEFAULT_APRS_FILTER: &str = "r/49.3678/16.1144/999999";
// pub const DEFAULT_APRS_FILTER: &str = "m/200";

pub const AIRCRAFT_REGEX: &str = "^([A-Z]{3})(.{6}).+?/([0-9]{6})h(.+)([NS]).(.+?)([EW])(.{1})(\\d{3})/(\\d{3})/A=([0-9]+).+?id(.{2})(.{6}).+?([+-].+)fpm.([+-].+)rot";
// pub const AIRCRAFT_REGEX_FL: &str = "FL([0-9.]+)";
