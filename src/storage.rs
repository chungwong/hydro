use std::sync::{Arc, Mutex};

use embedded_svc::storage::RawStorage;
use esp_idf_svc::{nvs::EspDefaultNvs, nvs_storage::EspNvsStorage};
use log::info;

use crate::STORAGE_NAME;

pub(crate) trait StorageBase {
    fn get(&self, _: &str) -> Option<String>;
    fn put(&self, _: &str, _: &[u8]) -> anyhow::Result<bool>;
}

impl StorageBase for Arc<Mutex<EspNvsStorage>> {
    fn get(&self, name: &str) -> Option<String> {
        let nvs = self.lock().unwrap();

        let len: usize = match nvs.len(name) {
            Ok(len) => len.unwrap_or(0),
            Err(err) => {
                info!("Key {} read returned {}", name, err);
                0
            }
        };

        let mut buf = vec![0u8; len];

        match nvs.get_raw(name, &mut buf) {
            Ok(_) => String::from_utf8(buf).ok(),
            Err(_) => None,
        }
    }

    fn put(&self, name: &str, buf: &[u8]) -> anyhow::Result<bool> {
        let mut nvs = self.lock().unwrap();
        Ok(nvs.put_raw(name, buf)?)
    }
}

pub(crate) struct Storage(pub(crate) Arc<Mutex<EspNvsStorage>>);

impl Storage {
    pub(crate) fn new(default_nvs: Arc<EspDefaultNvs>) -> anyhow::Result<Self> {
        Ok(Self(Arc::new(Mutex::new(EspNvsStorage::new_default(
            default_nvs,
            STORAGE_NAME,
            true,
        )?))))
    }
}
