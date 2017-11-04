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

use num_cpus;
use rand::{Rng, StdRng};
use scoped_threadpool::Pool;
use std::io::Write;
use striking_db::Store;

const OPERATIONS: u32 = 300_000;

fn inserts(store: &Store, id: u16) {
    let mut rng = StdRng::new().expect("Creating RNG failed");
    let mut key = [0; 32];
    let mut val = [0; 32];

    for i in 0..OPERATIONS {
        let mut key_slice = &mut key[..];
        let mut val_slice = &mut val[..];

        write!(&mut key_slice, "key_{}_{}", id, rng.next_u64()).unwrap();
        write!(&mut val_slice, "value_{}_{}", rng.next_u64(), rng.next_u64()).unwrap();

        store.insert(key_slice, val_slice).expect("Insertion failed!");
    }
}

pub fn run(store: Store) {
    let mut pool = Pool::new(num_cpus::get() as u32);

    pool.scoped(|scope| {
        let store = &store;

        println!("Running inserts...");
        for i in 0..32 {
            scope.execute(move || inserts(store, i));
        }
    });
}
