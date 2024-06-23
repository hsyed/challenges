use std::sync::Arc;
use std::time::Duration;

use cached::stores::ExpiringSizedCache;
use tokio::sync::RwLock;

use crate::protocol::{Question, ResourceRecord};

// The max TTL seconds allowed by the cache.
const MAX_TTL_SECONDS: u32 = 180;

pub struct DnsCacheValue {
    pub answers: Vec<ResourceRecord>,
}

impl DnsCacheValue {
    fn min_ttl(&self) -> Option<u64> {
        self.answers.iter()
            .min_by_key(|rr| rr.ttl)
            // If the TTL is greater than the max TTL allowed by the cache, use the max TTL.
            .map(|k| if k.ttl > MAX_TTL_SECONDS { MAX_TTL_SECONDS } else { k.ttl })
            .map(|ttl| Duration::from_secs(ttl as u64).as_millis() as u64)
    }
}

pub struct DnsCache {
    cache: RwLock<ExpiringSizedCache<Question, Arc<DnsCacheValue>>>,
}


impl DnsCache {
    // TODO consider upper bound on size to add LRU mechanics since the MAX_TTL is 24h
    pub fn new() -> DnsCache {
        DnsCache {
            cache: RwLock::new(ExpiringSizedCache::new(
                Duration::from_secs(MAX_TTL_SECONDS as u64).as_millis() as u64,
            )),
        }
    }

    pub async fn get(&self, question: &Question) -> Option<Arc<DnsCacheValue>> {
        let cache = self.cache.read().await;
        (*cache).get(question).map(|v| v.clone())
    }

    pub async fn set(&self, question: &Question, val: DnsCacheValue) {
        if let Some(ttl) = val.min_ttl() {
            let mut cache = self.cache.write().await;
            (*cache).insert_ttl((*question).clone(), Arc::new(val), ttl).expect("could not set key");
        }
    }
}