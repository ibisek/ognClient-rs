pub const DELAY_MS: u64 = 1*1000;

// const SERVER_ADDR: &str = "localhost:8888";
pub const SERVER_ADDR: &str = "aprs.glidernet.org:14580";   // port filtered
// pub const SERVER_ADDR: &str = "aprs.glidernet.org:10152";   // port unfiltered .. whatever that means

pub const DEFAULT_APRS_FILTER: &str = "filter r/49.3678/16.1144/99999";

pub const AIRCRAFT_REGEX1: &str = "^([A-Z]{3})(.{6}).+?/([0-9]{6})h([0-9.]+?)([NS]).([0-9.]+?)([EW])(.{1})(\\d{3})/(\\d{3})/A=([-0-9]+).+?id(.{2})(.{6}).([+-0123456789]+?)fpm.([+-0123456789]+?)rot";
pub const AIRCRAFT_REGEX2: &str = "^([A-Z]{3})(.{6}).+?/([0-9]{6})h([0-9.]+?)([NS]).([0-9.]+?)([EW])(.{1})(\\d{3})/(\\d{3})/A=([-0-9]+).+?id(.{2})(.{6}).([+-0123456789]+?)fpm";
pub const AIRCRAFT_REGEX3: &str = "^([A-Z]{3})(.{6}).+?/([0-9]{6})h([0-9.]+?)([NS]).([0-9.]+?)([EW])(.{1})(\\d{3})/(\\d{3})/A=([-0-9]+).+?id(.{2})(.{6})";
pub const AIRCRAFT_REGEX4: &str = "^([A-Z]{3})(.{6}).+?/([0-9]{6})h([0-9.]+?)([NS]).([0-9.]+?)([EW])(.{1})(\\d{3})/(\\d{3})/A=([-0-9]+)";
// pub const AIRCRAFT_REGEX_FL: &str = "FL([0-9.]+)";

pub const SKY_REGEX: &str = "^([A-Z]{3})([A-F0-9]{6}).+?([0-9]+)h([0-9.]+?)([NS])/([0-9.]+?)([EW])(.{1})(\\d{3})/(\\d{3})/A=([-0-9]+).+?id(.{2})(.{6}).([+-0123456789]+?)fpm";

pub const NEMO_REGEX: &str = "(.+?)>OGNEMO,qAS.+?/([0-9]+)h([0-9.]+?)([NS]).([0-9.]+?)([EW])(.{1})(\\d{3})/(\\d{3})/A=([-0-9]+).+?id(.{2})(.{6}).([+-0123456789]+?)fpm.([+-0123456789]+?)rot";
