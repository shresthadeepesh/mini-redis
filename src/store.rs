pub mod store {
    use std::{
        collections::HashMap,
        time::{Duration, SystemTime},
    };

    use log::info;

    #[derive(Debug)]
    pub struct Data {
        pub data: String,
        pub expires_in: SystemTime,
    }

    impl Data {
        pub fn is_expired(&self) -> bool {
            SystemTime::now() >= self.expires_in
        }
    }

    #[derive(Debug)]
    pub struct Store {
        pub data: HashMap<String, Data>,
    }

    pub trait RedisStore {
        fn set(&mut self, key: String, value: String, expires_in: u64);
        fn get(&self, key: String) -> String;
        fn delete(&mut self, key: String);
        fn cleanup_expired(&mut self);
    }

    impl RedisStore for Store {
        fn set(&mut self, key: String, value: String, expires_in: u64) {
            let data = Data {
                expires_in: SystemTime::now() + Duration::from_secs(expires_in),
                data: value,
            };

            self.data.insert(key, data);
        }

        fn get(&self, key: String) -> String {
            match self.data.get(&key) {
                Some(data) => {
                    if data.is_expired() {
                        info!("The data is expired.");
                        return String::from("");
                    }

                    data.data.clone()
                }
                None => {
                    info!("No data found for the key.");
                    String::from("")
                }
            }
        }

        fn delete(&mut self, key: String) {
            self.data.remove(&key);
        }

        fn cleanup_expired(&mut self) {
            let expired_keys: Vec<String> = self
                .data
                .iter()
                .filter(|(_, data)| data.is_expired())
                .map(|(key, _)| key.clone())
                .collect();

            for key in expired_keys {
                self.delete(key);
            }
        }
    }
}
