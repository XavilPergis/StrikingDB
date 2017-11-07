/*
 * test/perf.rs
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

use {flame, num_cpus};
use rand::{Rng, StdRng};
use scoped_threadpool::Pool;
use std::fs::File;
use std::io::Write;
use std::time::{Duration, Instant};
use striking_db::Store;

const OPERATIONS: u32 = 50_000;
const WRITE_RATIO: f32 = 0.30;

fn inserts(store: &Store, id: u32) {
    let mut rng = StdRng::new().expect("Creating RNG failed");
    let mut key = Vec::new();
    let mut val = Vec::new();

    for i in 0..OPERATIONS {
        key.clear();
        val.clear();

        write!(&mut key, "key_{}_{}", id, i).unwrap();
        write!(&mut val, "value_{}_{}_{}", i, rng.next_u64(), rng.next_u64()).unwrap();
        store.insert(key.as_slice(), val.as_slice()).expect("Insertion failed!");
    }
}

fn seq_reads(store: &Store, id: u32) {
    let mut key = Vec::new();
    let mut val = [0; 16];

    for i in 0..OPERATIONS {
        key.clear();

        write!(&mut key, "key_{}_{}", id, i).unwrap();
        store.lookup(key.as_slice(), &mut val[..]).expect("Sequential read failed!");
    }
}

fn rand_reads(store: &Store, id: u32) {
    let mut rng = StdRng::new().expect("Creating RNG failed");
    let mut key = Vec::new();
    let mut val = [0; 16];

    for _ in 0..OPERATIONS {
        key.clear();

        write!(&mut key, "key_{}_{}", id, rng.next_u32() % OPERATIONS).unwrap();
        store.lookup(key.as_slice(), &mut val[..]).expect("Random read failed!");
    }
}

fn updates(store: &Store, id: u32) {
    let mut key = Vec::new();

    for i in 0..OPERATIONS {
        key.clear();

        write!(&mut key, "key_{}_{}", id, i).unwrap();
        store.update(key.as_slice(), b"new value").expect("Update failed!");
    }
}

fn puts(store: &Store, _id: u32) {
    let mut rng = StdRng::new().expect("Creating RNG failed");
    let mut key = Vec::new();
    let mut val = Vec::new();

    for i in 0..OPERATIONS {
        key.clear();
        val.clear();

        write!(&mut key, "key_{}", rng.next_u64() % 512).unwrap();
        write!(&mut val, "val_{}_{}", i, rng.next_u64()).unwrap();
        store.put(key.as_slice(), val.as_slice()).expect("Put failed!");
    }
}

fn mixed(store: &Store, id: u32) {
    let mut rng = StdRng::new().expect("Creating RNG failed");
    let mut key = Vec::new();
    let mut val = Vec::new();

    for i in 0..OPERATIONS {
        key.clear();
        write!(&mut key, "key_{}_{}", id, i).unwrap();

        if rng.next_f32() < WRITE_RATIO {
            val.clear();
            write!(&mut val, "{}_{}", i, rng.next_u64()).unwrap();
            store.put(key.as_slice(), val.as_slice()).expect("Mixed put failed!");
        } else {
            store.lookup(key.as_slice(), val.as_mut_slice()).expect("Mixed lookup failed!");
        }
    }
}

fn deletes(store: &Store, id: u32) {
    let mut key = Vec::new();
    let mut val = [0; 0];

    for i in 0..OPERATIONS {
        key.clear();

        write!(&mut key, "key_{}_{}", id, i).unwrap();
        store.delete(key.as_slice(), &mut val[..]).expect("Update failed!");
    }
}

fn throughput(elapsed: Duration) {
    let mut seconds = elapsed.as_secs() as f64;
    seconds += (elapsed.subsec_nanos() as f64) / 1e9;
    let ops = OPERATIONS as f64 / seconds;

    println!("{:.2} operations / second", ops);
}

pub fn run(store: Store) {
    let cpus = num_cpus::get() as u32;
    let mut pool = Pool::new(cpus);

    {
        print!("Running inserts... ");
        flame::start_guard("inserts");
        let start = Instant::now();
        pool.scoped(|scope| {
            let store = &store;

            for i in 0..cpus {
                scope.execute(move || inserts(store, i));
            }
        });
        throughput(start.elapsed());
    }

    {
        print!("Running sequential reads... ");
        flame::start_guard("seq_read");
        let start = Instant::now();
        pool.scoped(|scope| {
            let store = &store;

            for i in 0..cpus {
                scope.execute(move || seq_reads(store, i));
            }
        });
        throughput(start.elapsed());
    }

    {
        print!("Running random reads... ");
        flame::start_guard("rand_read");
        let start = Instant::now();
        pool.scoped(|scope| {
            let store = &store;

            for i in 0..cpus {
                scope.execute(move || rand_reads(store, i));
            }
        });
        throughput(start.elapsed());
    }

    {
        print!("Running updates... ");
        flame::start_guard("updates");
        let start = Instant::now();
        pool.scoped(|scope| {
            let store = &store;

            for i in 0..cpus {
                scope.execute(move || updates(store, i));
            }
        });
        throughput(start.elapsed());
    }

    {
        print!("Running puts... ");
        flame::start_guard("puts");
        let start = Instant::now();
        pool.scoped(|scope| {
            let store = &store;

            for i in 0..cpus {
                scope.execute(move || puts(store, i));
            }
        });
        throughput(start.elapsed());
    }

    {
        print!("Running mixed ops... ");
        flame::start_guard("mixed");
        let start = Instant::now();
        pool.scoped(|scope| {
            let store = &store;

            for i in 0..cpus {
                scope.execute(move || mixed(store, i));
            }
        });
        throughput(start.elapsed());
    }

    {
        print!("Running deletes... ");
        flame::start_guard("deletes");
        let start = Instant::now();
        pool.scoped(|scope| {
            let store = &store;

            for i in 0..cpus {
                scope.execute(move || deletes(store, i));
            }
        });
        throughput(start.elapsed());
    }

    // Create report
    flame::dump_html(&mut File::create("flame-graph.html").unwrap()).unwrap();
}
