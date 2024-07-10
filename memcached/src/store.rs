use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::{Duration, Instant};

use moka::future::Cache;
use tokio::sync::{Mutex, MutexGuard};

use crate::protocol::{StorageCommand, StorageCommandResponse, StorageCommandType, Value};

struct Expiry;

/// expiry is derived from the ttl provided by the user on update and create.
impl moka::Expiry<String, Arc<Value>> for Expiry {
    fn expire_after_create(&self, _: &String, value: &Arc<Value>, _: Instant) -> Option<Duration> {
        Some(Duration::from_secs(value.exp_time as u64))
    }

    fn expire_after_update(&self, _: &String, value: &Arc<Value>, _: Instant, _: Option<Duration>) -> Option<Duration> {
        Some(Duration::from_secs(value.exp_time as u64))
    }
}

struct Store {
    cas_counter: AtomicU64,
    write_slots: Vec<Mutex<()>>,
    cache: Cache<String, Arc<Value>>,
}

impl Store {
    pub fn new() -> Store {
        let num_slots = num_cpus::get();
        let cas_counter = AtomicU64::new(0);
        let cache = Cache::builder().expire_after(Expiry {}).build();
        let write_slots = (0..num_slots).map(|_| Mutex::new(())).collect();

        Store {
            cache,
            write_slots,
            cas_counter,
        }
    }
    #[inline]
    async fn lock(&self, key: &String) -> MutexGuard<'_, ()> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        let slot = (hash % self.write_slots.len() as u64) as usize;
        self.write_slots[slot].lock().await
    }

    #[inline]
    fn next_cas(&self) -> u64 {
        self.cas_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) //TODO understand the SeqCst ordering
    }
}


pub(crate) struct StoreProcessor {
    store: Store,
}

impl StoreProcessor {
    pub(crate) fn new() -> StoreProcessor {
        let store = Store::new();

        StoreProcessor {
            store,
        }
    }

    pub(crate) async fn execute_storage_command(&self, mut args: StorageCommand) -> std::io::Result<StorageCommandResponse> {
        let _lock = self.store.lock(&args.key).await;

        return match args.command {
            StorageCommandType::Set => {
                self.do_insert(args).await;
                Ok(StorageCommandResponse::Stored)
            }
            StorageCommandType::Add => {
                if self.store.cache.get(&args.key).await.is_some() {
                    Ok(StorageCommandResponse::NotStored)
                } else {
                    self.do_insert(args).await;
                    Ok(StorageCommandResponse::Stored)
                }
            }
            StorageCommandType::Replace => {
                if self.store.cache.get(&args.key).await.is_none() {
                    Ok(StorageCommandResponse::NotStored)
                } else {
                    self.do_insert(args).await;
                    Ok(StorageCommandResponse::Stored)
                }
            }
            StorageCommandType::Prepend => {
                if let Some(val) = self.store.cache.get(&args.key).await {
                    args.data.extend_from_slice(&val.data);
                    self.do_insert(args).await;
                    Ok(StorageCommandResponse::Stored)
                } else {
                    Ok(StorageCommandResponse::NotStored)
                }
            }
            StorageCommandType::Append => {
                if let Some(val) = self.store.cache.get(&args.key).await {
                    args.data.reserve(args.data.len());
                    args.data.splice(0..0, val.data.iter().cloned());
                    self.do_insert(args).await;
                    Ok(StorageCommandResponse::Stored)
                } else {
                    Ok(StorageCommandResponse::NotStored)
                }
            }
        };
    }

    async fn do_insert(&self, args: StorageCommand) {
        let value = Arc::new(Value {
            flags: args.flags,
            exp_time: args.exp_time,
            data: args.data,
            cas: self.store.next_cas(),
        });
        self.store.cache.insert(args.key, value).await
    }

    pub(crate) async fn get(&self, key: &str) -> Option<Arc<Value>> { self.store.cache.get(key).await }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO:
    // 1. verify CAS.

    #[tokio::test]
    async fn test_processor_storage_set_add_replace() -> std::io::Result<()> {
        let processor = StoreProcessor::new();

        { // tests an add against a key that does not exist
            let command = StorageCommand {
                command: StorageCommandType::Add,
                key: "key".to_string(),
                exp_time: 60,
                data: b"value1".to_vec(),
                flags: 0,
                byte_count: 0,
                no_reply: false,
            };
            let res = processor.execute_storage_command(command).await?;
            assert_eq!(StorageCommandResponse::Stored, res);
            let res = processor.get(&"key".to_string()).await.unwrap();
            assert_eq!(b"value1".to_vec(), res.data);
        }

        { // tests an add against a key that already exists, should not overwrite
            let command = StorageCommand {
                command: StorageCommandType::Add,
                key: "key".to_string(),
                exp_time: 60,
                data: b"value2".to_vec(),
                no_reply: false,
                byte_count: 0,
                flags: 0,
            };
            let response = processor.execute_storage_command(command).await?;
            assert_eq!(StorageCommandResponse::NotStored, response);
            let res = processor.get(&"key".to_string()).await.unwrap();
            assert_eq!(b"value1".to_vec(), res.data);
        }

        { // tests a set against a key that already exists, should overwrite
            let command = StorageCommand {
                command: StorageCommandType::Set,
                key: "key".to_string(),
                exp_time: 60,
                data: b"value3".to_vec(),
                byte_count: 0,
                flags: 0,
                no_reply: false,
            };
            let res = processor.execute_storage_command(command).await?;
            assert_eq!(res, StorageCommandResponse::Stored);
            let res = processor.get(&"key".to_string()).await.unwrap();
            assert_eq!(b"value3".to_vec(), res.data);
        }

        { // replace an unknown key
            let command = StorageCommand {
                command: StorageCommandType::Replace,
                key: "key-unknown".to_string(),
                exp_time: 60,
                data: b"value4".to_vec(),
                byte_count: 0,
                flags: 0,
                no_reply: false,
            };

            let res = processor.execute_storage_command(command).await?;
            assert_eq!(res, StorageCommandResponse::NotStored);
            assert!(processor.get(&"key-unknown".to_string()).await.is_none());
        }

        { // replace an existing key
            let command = StorageCommand {
                command: StorageCommandType::Replace,
                key: "key".to_string(),
                exp_time: 60,
                data: b"value5".to_vec(),
                no_reply: false,
                byte_count: 0,
                flags: 0,
            };
            let res = processor.execute_storage_command(command).await?;
            assert_eq!(res, StorageCommandResponse::Stored);
            let res = processor.get(&"key".to_string()).await.unwrap();
            assert_eq!(b"value5".to_vec(), res.data);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_processor_storage_append_prepend() -> std::io::Result<()> {
        let processor = StoreProcessor::new();

        { // append and prepend to non-existing keys
            assert_eq!(StorageCommandResponse::NotStored,
                       processor.execute_storage_command(
                           StorageCommand {
                               command: StorageCommandType::Prepend,
                               key: "key-unknown".to_string(),
                               exp_time: 60,
                               data: b"unknown".to_vec(),
                               byte_count: 0,
                               flags: 0,
                               no_reply: false,
                           }).await?
            );
            assert_eq!(StorageCommandResponse::NotStored,
                       processor.execute_storage_command(
                           StorageCommand {
                               command: StorageCommandType::Append,
                               key: "key-unknown".to_string(),
                               exp_time: 60,
                               data: b"unknown".to_vec(),
                               byte_count: 0,
                               flags: 0,
                               no_reply: false,
                           }).await?
            );
        }

        { // create a key
            let command = StorageCommand {
                command: StorageCommandType::Set,
                key: "key".to_string(),
                exp_time: 60,
                data: b"b".to_vec(),
                byte_count: 0,
                flags: 0,
                no_reply: false,
            };
            let res = processor.execute_storage_command(command).await?;
            assert_eq!(res, StorageCommandResponse::Stored);
        }

        // prepend to the key
        {
            let command = StorageCommand {
                command: StorageCommandType::Prepend,
                key: "key".to_string(),
                exp_time: 60,
                data: b"a ".to_vec(),
                byte_count: 0,
                flags: 0,
                no_reply: false,
            };

            let res = processor.execute_storage_command(command).await?;
            assert_eq!(res, StorageCommandResponse::Stored);
            let res = processor.get(&"key".to_string()).await.unwrap();
            assert_eq!(b"a b".to_vec(), res.data);
        }

        // append to the key
        {
            let command = StorageCommand {
                command: StorageCommandType::Append,
                key: "key".to_string(),
                exp_time: 60,
                data: b" c".to_vec(),
                byte_count: 0,
                flags: 0,
                no_reply: false,
            };

            let res = processor.execute_storage_command(command).await?;
            assert_eq!(res, StorageCommandResponse::Stored);
            let res = processor.get(&"key".to_string()).await.unwrap();
            assert_eq!(b"a b c".to_vec(), res.data);
        }
        Ok(())
    }
}