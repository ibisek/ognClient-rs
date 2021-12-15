pub const DELAY_MS: u64 = 1*1000;

// const SERVER_ADDR: &str = "localhost:8888";
pub const SERVER_ADDR: &str = "aprs.glidernet.org:14580";   // port filtered
// pub const SERVER_ADDR: &str = "aprs.glidernet.org:10152";   // port unfiltered .. whatever that means

// pub const HANDSHAKE: &str = "user blume pass -1 vers rustBook 0.1 filter r/49.3678/16.1144/5000";
pub const HANDSHAKE: &str = "user blume pass -1 vers rustBook 0.1 filter r/49.3678/16.1144/999999";

pub const HANDSHAKE_TEMPLATE: &str = "user {} pass -1 vers rustClient 0.1 filter {}";   // user, filter
pub const HANSHAKE_FILTER_TEMPLATE: &str = "r/{:.f4}/{:.f4}/{:i}";    // lat, lon, range [km]
pub const HANSHAKE_FILTER_NONE: &str = "r/49.3678/16.1144/999999";

pub const AIRCRAFT_REGEX: &str = "^([A-Z]{3})(.{6}).+?/([0-9]{6})h(.+)([NS]).(.+?)([EW])(.{1})(\\d{3})/(\\d{3})/A=([0-9]+).+?id(.{2})(.{6}).+?([+-].+)fpm.([+-].+)rot";
pub const AIRCRAFT_REGEX_FL: &str = "FL([0-9.]+)";
