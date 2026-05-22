use std::{collections::HashMap, sync::Mutex, time::{Duration, Instant}};

pub struct MemoryCache {
    ttl: Duration,
    entries: Mutex<HashMap<String, CacheEntry>>,
}

struct CacheEntry {
    body: String,
    expires_at: Instant,
}

impl MemoryCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            entries: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let mut entries = self.entries.lock().expect("cache mutex poisoned");
        let now = Instant::now();

        match entries.get(key) {
            Some(entry) if entry.expires_at > now => Some(entry.body.clone()),
            Some(_) => {
                entries.remove(key);
                None
            }
            None => None,
        }
    }

    pub fn set(&self, key: String, body: String) {
        let mut entries = self.entries.lock().expect("cache mutex poisoned");

        entries.insert(
            key,
            CacheEntry {
                body,
                expires_at: Instant::now() + self.ttl,
            },
        );
    }
}
