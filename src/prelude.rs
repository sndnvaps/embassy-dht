// Copyright 2024 Developers of the embassy-dht project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Convenience re-export of common members
//!
//! Like the standard library's prelude, this module simplifies importing of
//! common items. Unlike the standard prelude, the contents of this module must
//! be imported manually:
//! ```
//! use embassy_dht::prelude::*;
//! use embassy_time::Delay;
//! let p = embassy_rp::init(Default::default());
//! let mut dht_pin = DHT11::new(p.PIN_17, Delay);
//! # let mut dht_pin = DHT22::new(p.PIN_17, Delay);
//! let dht_reading = dht_pin.read().unwrap();
//! let (temp, humi) = (dht_reading.get_temp(), dht_reading.get_hum());
//! #
//! ```

pub use crate::{Reading, SensorError};

#[cfg(feature = "dht11")]
pub use crate::dht11::DHT11;

#[cfg(feature = "dht22")]
pub use crate::dht22::DHT22;

#[cfg(feature = "dht20")]
pub use crate::dht20::DHT20;

#[cfg(feature = "embedded_alloc")]
pub use crate::DhtValueString;
