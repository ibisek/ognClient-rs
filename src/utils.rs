use chrono::Utc;
use regex::Captures;

pub fn now() -> String {
    let now = Utc::now();
    format!("{}", now.format("%Y-%m-%d %H:%M:%S%.3f"))
}


// safely parses out a string from regex Captures
pub fn from_caps<'a>(caps: &'a Captures<'a>, i: usize, default: &'a str) -> &'a str {
    match caps.get(i) {
        Some(m) => m.as_str(),
        None => default,
    }
}

pub fn from_caps_float<'a>(caps: &'a Captures<'a>, i: usize, default: f64) -> f64 {
    match caps.get(i) {
        Some(m) => {
            let f: f64 = match m.as_str().parse() {
                Ok(val) => val,
                Err(_) => default,
            };
            f
        },
        None => default,
    }
}

pub fn from_caps_int<'a>(caps: &'a Captures<'a>, i: usize, default: i64) -> i64 {
    match caps.get(i) {
        Some(m) => {
            let f: i64 = match m.as_str().parse() {
                Ok(val) => val,
                Err(_) => default,
            };
            f
        },
        None => default,
    }
}
