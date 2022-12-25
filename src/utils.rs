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

