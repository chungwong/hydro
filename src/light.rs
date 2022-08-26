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
                            7..=21 => {
                                info!("turning off light {:?}", self.pin.set_low());
                            }
                            _ => {
                                info!("turning on light {:?}", self.pin.set_high());
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
