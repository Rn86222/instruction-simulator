use std::fs::File;
use std::io::{self, BufRead};

pub fn load_sld_file(file_path: &str) -> Vec<String> {
    let mut sld_vec = Vec::new();

    if let Ok(file) = File::open(file_path) {
        let reader = io::BufReader::new(file);
        for line in reader.lines().flatten() {
            let iter = line.split_whitespace();
            for token in iter {
                sld_vec.push(token.to_string());
            }
        }
    } else {
        eprintln!("Warning: failed to open sld file (dismiss if you don't need it).");
    }

    sld_vec
}
