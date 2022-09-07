use core::{num::ParseIntError, str::FromStr, time::Duration};
use log::info;
use std::{
    sync::{Arc, Mutex},
    thread,
};

use std::fmt::Debug;

use embedded_hal::digital::blocking::OutputPin;

use time::{format_description::well_known::Rfc3339, OffsetDateTime};
pub(crate) struct Light<T>
where
    T: OutputPin + Send + 'static,
{
    pub(crate) pin: T,
    pub(crate) hours: LightHours,
}

impl<T> Light<T>
where
    T: OutputPin + Send + 'static,
{
    pub(crate) fn new(pin: T, hours: LightHours) -> Self {
        Self { pin, hours}
    }

    /// Mapping for UTC and local time zones
    /// UTC Local(+10) Local(+11)
    ///  0     10         11
    ///  1     11         12
    ///  2     12         13
    ///  3     13         14
    ///  4     14         15
    ///  5     15         16
    ///  6     16         17
    ///  7     17         18
    ///  8     18         19
    ///  9     19         20
    /// 10     20         21
    /// 11     21         22
    /// 12     22         23
    /// 13     23          0
    /// 14      2          1
    /// 15      3          2
    /// 16      4          3
    /// 17      5          4
    /// 18      6          5
    /// 19      7          6
    /// 20      8          7
    /// 21      9          8
    /// 22     10          9
    /// 23     11         10
    pub(crate) fn toggle(light: Arc<Mutex<Light<T>>>) {
        thread::spawn(move || loop {
            let utc_now = OffsetDateTime::now_utc();

            info!(
                "{:?} checking time to toggle light",
                utc_now.format(&Rfc3339)
            );

            let mut light = light.lock().unwrap();

            let sleep_secs = if utc_now.year() == 1970 {
                // sntp is not synced yet
                1
            } else {
                let hour = utc_now.hour();

                if light.hours.0.contains(&hour) {
                    info!("turning on light {:?}", light.pin.set_high());
                } else {
                    info!("turning off light {:?}", light.pin.set_low());
                }
                3600
            };

            thread::sleep(Duration::from_secs(sleep_secs));
        });
    }
}

#[derive(Default, Debug)]
pub(crate) struct LightHours(pub(crate) Vec<u8>);

impl LightHours {
    pub(crate) fn to_html_options(&self) -> String {
        (0u8..=23u8)
            .map(|h| {
                let selected = if self.0.contains(&h) { "selected" } else { "" };

                format!("<option value=\"{h}\" {selected}>{h}</option>")
            })
            .collect::<Vec<String>>()
            .join("")
    }
}

impl FromStr for LightHours {
    type Err = ParseIntError;

    fn from_str(hours: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            hours
                .split(',')
                .map(str::trim)
                .map(u8::from_str)
                .collect::<Result<Vec<u8>, Self::Err>>()?,
        ))
    }
}
