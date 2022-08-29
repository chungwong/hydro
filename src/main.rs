mod light;
mod server;
mod wifi;

use core::time::Duration;
use std::{
    sync::{Arc, Mutex},
    thread,
};

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{
    sntp,
    netif::EspNetifStack, nvs::EspDefaultNvs, sysloop::EspSysLoopStack,
};
use esp_idf_svc::nvs_storage::EspNvsStorage;


use crate::light::Light;

const SSID: &str = env!("WIFI_SSID");
const PASS: &str = env!("WIFI_PASS");

fn main() -> anyhow::Result<()> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let netif_stack = Arc::new(EspNetifStack::new()?);
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    let default_nvs = Arc::new(EspDefaultNvs::new()?);

    // Connect to the Wi-Fi network
    let _wifi = match wifi::wifi(netif_stack.clone(), sys_loop_stack.clone(), default_nvs.clone(), SSID, PASS) {
        Ok(inner) => inner,
        Err(err) => {
            anyhow::bail!("could not connect to Wi-Fi network: {:?}", err)
        }
    };

    let _sntp = sntp::EspSntp::new_default()?;

    let storage = Arc::new(Mutex::new(EspNvsStorage::new_default(default_nvs.clone(), "my_area", true)?));
    let _server = server::start(storage.clone());

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
