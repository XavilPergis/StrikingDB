/*
 * cache.rs
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

use page::{Page, PageId};
use std::collections::{BTreeMap, VecDeque};
use std::borrow::Borrow;
use std::fmt;
use strand::Strand;
use super::Result;

pub type PageCleanupFn = FnMut(PageId, &mut Page) -> Result<()>;

#[derive(Debug)]
pub struct PageCache(LruCache<PageId, Page, PageCleanupFn>);

impl PageCache {
    pub fn new(strand: &Strand) -> Self {
        const CACHE_CAPACITY: usize = 512;

        PageCache(
            LruCache::with_capacity(
                Box::new(|id, page| page.flush(&mut strand, id)),
                CACHE_CAPACITY,
            )
        )
    }
}

pub struct LruCache<K, V, F>
where
    K: Ord + Clone,
    F: FnMut(K, &mut V) -> Result<()>,
    F: ?Sized,
{
    items: BTreeMap<K, V>,
    list: VecDeque<K>,
    capacity: usize,
    purge_callback: Box<F>,
}

impl<K, V, F> LruCache<K, V, F>
where
    K: Ord + Clone,
    F: FnMut(K, &mut V) -> Result<()>,
    F: ?Sized,
{
    pub fn new(purge_callback: Box<F>) -> Self {
        Self::with_capacity(purge_callback, 64)
    }

    pub fn with_capacity(purge_callback: Box<F>, capacity: usize) -> Self {
        assert_ne!(capacity, 0, "Capacity cannot be zero");

        LruCache {
            items: BTreeMap::new(),
            list: VecDeque::with_capacity(capacity + 1),
            capacity: capacity,
            purge_callback: purge_callback,
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.items.contains_key(&key) {
            Self::update_key(&mut self.list, &key);
        } else {
            self.prune();
            self.list.push_back(key.clone());
        }

        self.items.insert(key, value)
    }

    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        self.list.retain(
            |k| *k.borrow() < *key || *k.borrow() > *key,
        );
        self.items.remove(key)
    }

    pub fn clear(&mut self) {
        for (key, value) in self.mut_iter() {
            (*self.purge_callback)(key.clone(), value);
        }

        self.items.clear();
        self.list.clear();
    }

    pub fn peek<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        self.items.get(key)
    }

    pub fn get<Q: ?Sized>(&mut self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        self.prune();
        let list = &mut self.list;

        self.items.get_mut(key).map(|result| {
            Self::update_key(list, key);
            &result
        })
    }

    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        self.prune();
        let list = &mut self.list;

        self.items.get_mut(key).map(|result| {
            Self::update_key(list, key);
            &mut result
        })
    }

    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        self.items.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.len() == 0
    }

    fn prune(&mut self) {
        while self.items.len() >= self.capacity {
            let _ = self.list.pop_front().map(|key| {
                assert!(self.items.remove(&key).is_some())
            });
        }
    }

    fn update_key<Q: ?Sized>(list: &mut VecDeque<K>, key: &Q)
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        if let Some(pos) = list.iter().position(|k| k.borrow() == key) {
            let k = list.remove(pos).unwrap();
            list.push_back(k);
        }
    }
}

impl<K, V, F> fmt::Debug for LruCache<K, V, F>
    where
        K: Ord + Clone,
        F: FnMut(K, &mut V) -> Result<()>,
        F: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(&mut f, "LruCache {{")?;
        writeln!(&mut f, "    items:  {:?}", self.items)?;
        writeln!(&mut f, "    list: {:?}", self.list)?;
        writeln!(&mut f, "    capacity: {:?}", self.capacity)?;
        writeln!(&mut f, "}}")?;

        Ok(())
    }
}

impl<K, V, F> Drop for LruCache<K, V, F>
    where
        K: Ord + Clone,
        F: FnMut(K, &mut V) -> Result<()>,
        F: ?Sized,
{
    fn drop(&mut self) {
        self.clear();
    }
}
