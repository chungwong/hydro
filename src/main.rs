mod light;
mod server;
mod storage;
mod wifi;

use core::time::Duration;
use std::{
    sync::{Arc, Mutex},
    thread,
};

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{netif::EspNetifStack, nvs::EspDefaultNvs, sntp, sysloop::EspSysLoopStack};

use crate::{
    light::Light,
    storage::{Storage, StorageBase},
};

const SSID: Option<&str> = option_env!("WIFI_SSID");
const PASS: Option<&str> = option_env!("WIFI_PASS");

const STORAGE_NAME: &str = env!("CARGO_PKG_NAME");

fn main() -> anyhow::Result<()> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let netif_stack = Arc::new(EspNetifStack::new()?);
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    let default_nvs = Arc::new(EspDefaultNvs::new()?);

    let storage = Storage::new(default_nvs.clone())?;

    let ssid: String = SSID.map_or_else(
        || storage.0.get("WIFI_SSID").unwrap_or_default(),
        |s| s.to_string(),
    );

    let pass: String = PASS.map_or_else(
        || storage.0.get("WIFI_PASS").unwrap_or_default(),
        |s| s.to_string(),
    );

    // Connect to the Wi-Fi network
    let _wifi = match wifi::wifi(netif_stack, sys_loop_stack, default_nvs, &ssid, &pass) {
        Ok(inner) => inner,
        Err(err) => {
            anyhow::bail!("could not connect to Wi-Fi network: {:?}", err)
        }
    };

    let _sntp = sntp::EspSntp::new_default()?;

    let _server = server::start(&storage);

    let peripherals = Peripherals::take().unwrap();

    let light = Arc::new(Mutex::new(Light::new(
        peripherals.pins.gpio20.into_output()?,
        vec![0..=11, 20..=23],
    )));

    Light::toggle(light);

    loop {
        thread::sleep(Duration::from_secs(5));
    }
}
