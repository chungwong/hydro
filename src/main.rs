#![feature(slice_group_by)]

mod light;
mod server;
mod storage;
mod wifi;

use core::str::FromStr;
use core::time::Duration;

use std::sync::{Arc, Mutex};

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use embedded_hal::digital::blocking::OutputPin;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{netif::EspNetifStack, nvs::EspDefaultNvs, sntp, sysloop::EspSysLoopStack};

use hydro::button::Button;

use crate::{
    light::{Light, LightCheck, LightHours},
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

    // let peripherals = Arc::new(Mutex::new(Peripherals::take().unwrap()));
    let peripherals = Peripherals::take().unwrap();

    let hours = if let Some(ref hrs) = storage.0.get("LIGHT_HOURS") {
        if let Ok(hours) = LightHours::from_str(hrs) {
            hours
        } else {
            LightHours::default()
        }
    } else {
        LightHours::default()
    };

    let gpio20 = peripherals.pins.gpio20;

    let light = Arc::new(Mutex::new(Light::new(gpio20.into_output()?, hours)));

    light.lock().unwrap().pin.set_low()?;

    light.toggle();

    let mut boot_button = Button::new(peripherals.pins.gpio9.into_input()?)
        .set_long_press_duration(Duration::from_secs(1));

    boot_button.set_short_action(Box::new(|_pin| {
        dbg!("new short press callback");
    }));

    boot_button.set_long_action(Box::new(|_pin| {
        dbg!("new long press callback");
    }));

    loop {
        boot_button.poll();
    }
}
