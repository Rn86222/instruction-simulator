use std::fs::File;
use std::io::{self, BufRead};

use fxhash::FxHashMap;

use crate::types::Address;

pub fn load_label_map_file(file_path: &str) -> FxHashMap<Address, String> {
    let mut pc_to_label_map = FxHashMap::default();

    if let Ok(file) = File::open(file_path) {
        let reader = io::BufReader::new(file);
        for line in reader.lines().flatten() {
            let (label, addr) = line.split_at(line.find(' ').unwrap());
            // if label.starts_with("beq")
            //     || label.starts_with("bge")
            //     || label.starts_with("ble")
            //     || label.starts_with("fble")
            //     || label.starts_with("fbeq")
            // {
            //     continue;
            // }
            let label = label.split('.').next().unwrap();
            let addr = addr.trim().parse::<Address>().unwrap();
            pc_to_label_map.insert(addr, label.to_string());
        }
    } else {
        eprintln!("Warning: failed to open sld file (dismiss if you don't need it).");
    }

    pc_to_label_map
}
