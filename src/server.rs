use std::{
    sync::{Arc, Mutex},
};
use embedded_svc::storage::{RawStorage, StorageBase};
use esp_idf_svc::nvs_storage::EspNvsStorage;
use esp_idf_svc::nvs::EspDefaultNvs;

use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use embedded_svc::httpd::registry::*;
use esp_idf_svc::httpd::{Server, ServerRegistry};

use log::*;

pub fn start(storage: Arc<Mutex<EspNvsStorage>>) -> anyhow::Result<Server> {
    let storage2 = storage.clone();
    let server = ServerRegistry::new()
        .at("/")
        .get(|_| Ok(format!("{:?}", OffsetDateTime::now_utc().format(&Rfc3339)).into()))?
        .at("/get_nvs")
        .get(move |_| {
            let name = "who";

            let storage = storage.lock().unwrap();

            let len: usize = match storage.len(name) {
                Ok(len) => match len {
                    Some(len) => len,
                    None => 0 as usize
                },
                Err(err) => {
                    info!("Key {} read returned {}", name, err);
                    0 as usize
                },
            };

            let mut a = vec![0u8; len];
            let buf: &mut [u8] = &mut a;

            match storage.get_raw(name, buf) {
                Ok(val) => {
                    dbg!(val);
                    dbg!(&buf);
                    Ok(std::str::from_utf8(&buf).unwrap().into())
                }, 
                Err(err) => {
                    info!("Key {} read returned {}", name, err);
                    Ok(format!("{:?}", err).into())
                },
            }
        })?
        .at("/put_nvs")
        .get(move |r| {
            let val =
                match r.query_string() {
                    Some(s) => s,
                    _ => String::from("default")
                };

            let name = "who";
            let mut storage2 = storage2.lock().unwrap();
            let content = storage2.put_raw(name, val.as_bytes())?;
            Ok(format!("{:?}", content).into())
        })?;

    server.start(&Default::default())
}

