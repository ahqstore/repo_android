use std::fs;
use serde_json::from_str;
use structs::Metadata;

mod structs;
mod parser;

fn main() {
    let data = fs::read_to_string("./index.json").expect("Unable to read metadata");

    let parsed: Metadata = from_str(&data).expect("Parsing failed");

    // Reduces >30MB of memory usage
    drop(data);
    
    parser::parser(parsed);
}
