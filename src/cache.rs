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
use strand::Strand;
use super::{PAGE_SIZE, Result};

pub type PageCleanupFn = (FnMut(PageId, &mut Page) -> Result<()>);

#[derive(Debug)]
pub struct PageCache(LruCache<PageId, Page, PageCleanupFn>);

impl PageCache {
    pub fn new(strand: &Strand) -> Self {
        const CACHE_CAPACITY: usize = 512;

        let purge = |id, page| {
            if page.dirty() {
                strand.write(id * PAGE_SIZE, &page[..])?;
            }

            Ok(())
        };

        PageCache(LruCache::with_capacity(Box::new(purge), CACHE_CAPACITY))
    }
}

#[derive(Debug)]
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
    F: FnMut(K, &mut V),
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
        if self.map.contains_key(&key) {
            Self::update_key(&mut self.list, &key);
        } else {
            self.prune();
            self.list.push_back(key.clone());
        }

        self.map.insert(key, value).map(|pair| pair.0)
    }

    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        self.list.retain(
            |k| *k.borrow() < *key || *k.borrow() > *key,
        );
        self.map.remove(key)
    }

    pub fn clear(&mut self) {
        for (key, value) in self.iter_mut() {
            self.purge_callback(key.clone(), value);
        }

        self.map.clear();
        self.list.clear();
    }

    pub fn peek<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        self.map.get(key)
    }

    pub fn get<Q: ?Sized>(&mut self, key: &Q) -> Option<&V>
    where
        Key: Borrow<Q>,
        Q: Ord,
    {
        self.prune();
        let list = &mut self.list;

        self.map.get_mut(key).map(|result| {
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

        self.map.get_mut(key).map(|result| {
            Self::update_key(list, key);
            &mut result
        })
    }

    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        self.map.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.len() == 0
    }

    fn prune(&mut self) {
        while self.map.len() >= self.capacity {
            let _ = self.list.pop_front().map(|key| {
                assert!(self.map.remove(&key).is_some())
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

impl<K, V, F> Drop for LruCache<K, V, F>
    where
        K: Ord + Clone,
        F: FnMut(K, &mut V),
        F: ?Sized,
{
    fn drop(&mut self) {
        self.clear();
    }
}
