extern crate striking_db;

use std::{env, process};
use striking_db::{OpenOptions, Store};

fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        eprintln!("Usage: {} datastore-file-path", &args[0]);
        process::exit(1);
    }

    let path = &args[1];
    let options = {
        let mut options = OpenOptions::new();
        options.truncate();
        options
    };

    let store = Store::open(path, &options).expect("Opening datastore failed");
}
