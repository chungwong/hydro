mod wifi;

use anyhow::bail;
use log::*;
use std::{thread, time::Duration};
use time::OffsetDateTime;

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use embedded_svc::{ipv4, ping::Ping};
use esp_idf_svc::{ping, sntp};

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

    loop {
        // ping(ipv4::Ipv4Addr::new(142,250,70,238));
        dbg!(OffsetDateTime::now_utc().hour());

        thread::sleep(Duration::from_secs(5));
    }
}

fn ping(ip_settings: ipv4::Ipv4Addr) -> anyhow::Result<()> {
    info!("About to do some pings for {:?}", ip_settings);

    let ping_summary = ping::EspPing::default().ping(ip_settings, &Default::default())?;
    if ping_summary.transmitted != ping_summary.received {
        bail!("Pinging gateway {} resulted in timeouts", ip_settings);
    }

    info!("Pinging done");

    Ok(())
}
