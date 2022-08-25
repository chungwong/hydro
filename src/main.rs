mod server;
mod wifi;

use std::{thread, time::Duration};
use time::OffsetDateTime;

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use esp_idf_svc::sntp;

const SSID: &str = env!("WIFI_SSID");
const PASS: &str = env!("WIFI_PASS");

fn main() -> anyhow::Result<()> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // Connect to the Wi-Fi network
    let _wifi = match wifi::wifi(SSID, PASS) {
        Ok(inner) => inner,
        Err(err) => {
            anyhow::bail!("could not connect to Wi-Fi network: {:?}", err)
        }
    };

    let _sntp = sntp::EspSntp::new_default()?;

    let _server = server::start();

    loop {
        dbg!(OffsetDateTime::now_utc().hour());

        thread::sleep(Duration::from_secs(5));
    }
}
