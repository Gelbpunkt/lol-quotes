use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::ddragon::QuoteExport;

const QUOTES_JSON: &[u8] = include_bytes!("../quotes.json");

lazy_static! {
    pub static ref CHAMPIONS: HashMap<String, QuoteExport> =
        serde_json::from_slice(QUOTES_JSON).expect("Invalid JSON in quotes.json");
    pub static ref ALL_CHAMPIONS_STRING: String =
        CHAMPIONS.keys().fold(String::new(), |a, b| a + b + "\n");
}
