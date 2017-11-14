/*
 * test/test.rs
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

use striking_db::Store;

pub fn run(store: Store) {
    let mut value = [0; 16];

    store.insert(b"abc", b"000").expect("0 - insert");
    {
        let len = store.lookup(b"abc", &mut value[..]).expect("1 - lookup");
        assert_eq!(b"000", &value[..len]);
    }
    {
        let len = store.lookup(b"abc", &mut value[..]).expect("2 - lookup");
        assert_eq!(b"000", &value[..len]);
    }
    {
        let len = store.lookup(b"abc", &mut value[..]).expect("3 - lookup");
        assert_eq!(b"000", &value[..len]);
    }

    store.update(b"abc", b"111").expect("4 - update");
    {
        let len = store.lookup(b"abc", &mut value[..]).expect("5 - lookup");
        assert_eq!(b"111", &value[..len]);
    }
    {
        let len = store.lookup(b"abc", &mut value[..]).expect("6 - lookup");
        assert_eq!(b"111", &value[..len]);
    }

    store.insert(b"def", b"ABCDEF").expect("7 - insert");
    assert_eq!(store.exists(b"def"), true);
    {
        let len = store.delete(b"def", &mut value[..]).expect("8 - delete");
        assert_eq!(b"ABCDEF", &value[..len]);

        assert!(!store.exists(b"def"), "9 - exists after delete");
        store.lookup(b"def", &mut value[..]).expect_err("10 - lookup after delete");
    }
}
