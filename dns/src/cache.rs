use std::time::{Duration, SystemTime};

use cached::stores::ExpiringSizedCache;
use tokio::sync::RwLock;

use crate::protocol::{Question, ResourceRecord};

// TODO consider capping the the upper bound of cached ttl to MAX_TTL_SECONDS ?

// The max TTL seconds allowed by the cache.
const MAX_TTL_SECONDS: u32 = 1800; // 30 minutes

struct DnsCacheValue {
    answers: Vec<ResourceRecord>,
    inserted_at: SystemTime,
}


fn min_ttl(rr: &Vec<ResourceRecord>) -> Option<u64> {
    rr.iter()
        .min_by_key(|rr| rr.ttl)
        // If the TTL is greater than the max TTL allowed by the cache, use the max TTL.
        .map(|k| if k.ttl > MAX_TTL_SECONDS { MAX_TTL_SECONDS } else { k.ttl })
        .map(|ttl| Duration::from_secs(ttl as u64).as_millis() as u64)
}


pub struct DnsCache {
    cache: RwLock<ExpiringSizedCache<Question, DnsCacheValue>>,
}


impl DnsCache {
    // TODO consider upper bound on size
    pub fn new() -> DnsCache {
        DnsCache {
            cache: RwLock::new(ExpiringSizedCache::new(
                Duration::from_secs(MAX_TTL_SECONDS as u64).as_millis() as u64,
            )),
        }
    }

    pub async fn get(&self, question: &Question) -> Option<Vec<ResourceRecord>> {
        let cache = self.cache.read().await;
        cache.get(question).map(|v| {
            let mut answers = v.answers.clone();
            // return a copy of the answers with the TTLs adjusted.
            let elapsed = v.inserted_at.elapsed().unwrap().as_secs() as u32;
            for rr in &mut answers {
                rr.ttl = if elapsed < rr.ttl { rr.ttl - elapsed } else { 0 }
            }
            answers
        })
    }

    pub async fn set(&self, question: &Question, answers: &Vec<ResourceRecord>) {
        if let Some(ttl) = min_ttl(answers) {
            let mut cache = self.cache.write().await;
            cache.insert_ttl(
                question.clone(),
                DnsCacheValue {
                    answers: answers.clone(),
                    inserted_at: SystemTime::now(),
                }, ttl).expect("could not set key");
        }
    }
}