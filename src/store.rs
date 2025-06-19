extern crate lru;

use lru::LruCache;
use std::hash::Hash;
use std::num::NonZeroUsize;

pub struct Store<K, V> {
    size: usize,
    cache: LruCache<K, V>,
}

impl<K: Eq + Hash, V> Store<K, V> {
    pub fn new(size: usize) -> Store<K, V> {
        Store {
            size,
            cache: LruCache::new(NonZeroUsize::new(size).unwrap()),
        }
    }

    pub fn get(&mut self, key: K) -> &V {
        self.cache.get_mut(&key).unwrap()
    }

    pub fn put(&mut self, key: K, val: V) {
        self.cache.put(key, val);
    }

    pub fn delete(&mut self, key: K) {
        self.cache.pop(&key);
    }
}
