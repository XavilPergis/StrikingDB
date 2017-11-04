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
    let mut value = [0; 16];

    store.insert(b"abc", b"000").expect("1 - Insertion failed");
    {
        let len = store.lookup(b"abc", &mut value[..]).expect("1 - Lookup failed");
        assert_eq!(b"000", &value[..len]);
    }
    {
        let len = store.lookup(b"abc", &mut value[..]).expect("1 - Lookup failed");
        assert_eq!(b"000", &value[..len]);
    }
    {
        let len = store.lookup(b"abc", &mut value[..]).expect("1 - Lookup failed");
        assert_eq!(b"000", &value[..len]);
    }

    store.update(b"abc", b"111").expect("2 - Update failed");
    {
        let len = store.lookup(b"abc", &mut value[..]).expect("2 - Lookup failed");
        assert_eq!(b"111", &value[..len]);
    }
    {
        let len = store.lookup(b"abc", &mut value[..]).expect("2 - Lookup failed");
        assert_eq!(b"111", &value[..len]);
    }

    store.insert(b"def", b"ABCDEF").expect("3 - Insertion failed");
    {
        let len = store.delete(b"def", &mut value[..]).expect("3 - Delete failed");
        assert_eq!(b"ABCDEF", &value[..len]);

        store.lookup(b"def", &mut value[..]).expect_err("3 - Lookup succeeded");
    }
}
