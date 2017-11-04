/*
 * test/main.rs
 *
 * striking-db - Persistent key/value store for SSDs.
 * Copyright (c) 2017 Maxwell Duzen, Ammon Smith
 *
 * striking-db is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as
 * published by the Free Software Foundation, either version 2 of
 * the License, or (at your option) any later version.
 *
 * striking-db is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with striking-db.  If not, see <http://www.gnu.org/licenses/>.
 *
 */

extern crate num_cpus;
extern crate rand;
extern crate scoped_threadpool;
extern crate striking_db;

mod perf;
mod test;

use std::{env, process};
use striking_db::{OpenOptions, Store};

fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() != 3 {
        eprintln!("Usage: {} operation datastore-file-path", &args[0]);
        process::exit(1);
    }

    let path = &args[2];
    let options = {
        let mut options = OpenOptions::new();
        options.truncate();
        options
    };

    let store = Store::open(path, &options).expect("Opening datastore failed");

    let caller = match args[1].as_str() {
        "perf" => perf::run,
        "test" => test::run,
        name => {
            eprintln!("No such operation: {}", name);
            process::exit(1);
        }
    };

    caller(store);
}
