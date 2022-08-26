use core::time::Duration;
use log::info;
use std::thread;

use embedded_hal::digital::blocking::OutputPin;

use time::{format_description::well_known::Rfc3339, OffsetDateTime};
pub(crate) struct Light<T: OutputPin + Send> {
    pin: T,
}

impl<T: OutputPin + Send> Light<T> {
    pub(crate) fn new(pin: T) -> Self {
        Self { pin }
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
    pub(crate) fn toggle(&mut self) {
        thread::scope(|s| {
            s.spawn(|| {
                loop {
                    let utc_now = OffsetDateTime::now_utc();
                    info!(
                        "{:?} checking time to toggle light",
                        utc_now.format(&Rfc3339)
                    );

                    let sleep_secs = if utc_now.year() == 1970 {
                        // sntp is not synced yet
                        1
                    } else {
                        match utc_now.hour() {
                            0..=11 | 20..=23 => {
                                info!("turning on light {:?}", self.pin.set_high());
                            }
                            _ => {
                                info!("turning off light {:?}", self.pin.set_low());
                            }
                        };
                        3600
                    };

                    thread::sleep(Duration::from_secs(sleep_secs));
                }
            });
        });
    }
}
