use std::{sync::Arc, time::*};

use anyhow::bail;
use embedded_svc::wifi::{
    self, AccessPointConfiguration, AuthMethod, ClientConfiguration, ClientConnectionStatus,
    ClientIpStatus, ClientStatus,
    Configuration::{Client, Mixed},
    Wifi as _,
};
use esp_idf_svc::{
    netif::EspNetifStack, nvs::EspDefaultNvs, sysloop::EspSysLoopStack, wifi::EspWifi,
};
use log::info;

#[allow(unused)]
pub(crate) struct Wifi {
    pub(crate) esp_wifi: EspWifi,
}

impl Wifi {
    pub(crate) fn disable_ap(&mut self) -> anyhow::Result<()> {
        if let Ok(Mixed(client_configuration, _)) = self.esp_wifi.get_configuration() {
            info!("Change Wifi to ClientConfiguration");
            self.esp_wifi
                .set_configuration(&Client(client_configuration))?;
        }

        Ok(())
    }
}

pub(crate) fn wifi(
    netif_stack: Arc<EspNetifStack>,
    sys_loop_stack: Arc<EspSysLoopStack>,
    default_nvs: Arc<EspDefaultNvs>,
    ssid: &str,
    psk: &str,
) -> anyhow::Result<Wifi> {
    let mut auth_method = AuthMethod::WPA2Personal;
    if ssid.is_empty() {
        anyhow::bail!("missing WiFi name")
    }
    if psk.is_empty() {
        auth_method = AuthMethod::None;
        info!("Wifi password is empty");
    }

    let mut wifi = EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?;

    info!("Searching for Wifi network {}", ssid);

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == ssid);

    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            ssid, ours.channel
        );
        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            ssid
        );
        None
    };

    info!("setting Wifi configuration");
    wifi.set_configuration(&Mixed(
        ClientConfiguration {
            ssid: ssid.into(),
            password: psk.into(),
            channel,
            auth_method,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "ESP32 C3".into(),
            password: "admin".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    ))?;

    wifi.wait_status_with_timeout(Duration::from_secs(10), |status| !status.is_transitional())
        .map_err(|e| anyhow::anyhow!("Unexpected Wifi status: {:?}", e))?;

    info!("getting Wifi status");

    let status = wifi.get_status();

    if let wifi::Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(_))),
        _,
    ) = status
    {
        info!("Wifi connected!");
    } else {
        bail!("Unexpected Wifi status: {:?}", status);
    }

    let wifi = Wifi { esp_wifi: wifi };

    Ok(wifi)
}
