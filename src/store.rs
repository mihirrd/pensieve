extern crate lru;

use lru::LruCache;
use std::num::NonZeroUsize;


pub struct Store {
    cache: LruCache<String, String>,
}

impl Store {
    pub fn new(size: usize) -> Store {
        Store {
            cache: LruCache::new(NonZeroUsize::new(size).unwrap()),
        }
    }

    pub fn get(&mut self, key: String) -> String {
        let val = self.cache.get(&key);
        match val {
            Some(val) => val.to_string(),
            None => "".to_string()
        }
    }

    pub fn put(&mut self, key: String, val: String) {
        self.cache.put(key, val);
    }

    pub fn delete(&mut self, key: String) {
        self.cache.pop(&key);
    }
}
