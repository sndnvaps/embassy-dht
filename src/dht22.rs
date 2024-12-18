// Copyright 2024 Developers of the embassy-dht project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! DHT22 functionality
//!

use embassy_rp::gpio::{Flex, Pin};
use embassy_rp::Peripheral;
use embedded_hal::delay::DelayNs;

use crate::wait_for_state;
use crate::{F32Utils, Reading, SensorError};

pub struct DHT22<'a, D> {
    pub pin: Flex<'a>,
    pub delay: D,
}

impl<'a, D> DHT22<'a, D>
where
    D: DelayNs,
{
    pub fn new(pin: impl Peripheral<P = impl Pin> + 'a, delay: D) -> Self {
        let pin = Flex::new(pin);
        Self { pin, delay }
    }

    pub fn read(&mut self) -> Result<Reading<f32, f32>, SensorError> {
        let data = self.read_raw()?;

        let raw_temp: u16 = (data[2] as u16) << 8 | data[3] as u16;

        // If the first bit of the 16bit word is set the temp. is negative
        // Didn't have negative temps around to test it,
        // so the conversion might be wrong as there are numerous different
        // pieces of info on the subject over the Internet.
        // Maybe will update it when the winter comes :)
        let temp: f32 = match raw_temp & 0x8000 == 1 {
            true => -0.1 * (raw_temp & 0x7fff) as f32,
            false => 0.1 * raw_temp as f32,
        };

        let raw_hum: u16 = (data[0] as u16) << 8 | data[1] as u16;
        let hum: f32 = 0.1 * raw_hum as f32;

        let temp = temp.round_fixed(2);
        let hum = hum.round_fixed(2);

        Ok(Reading { temp, hum })
    }

    fn read_raw(&mut self) -> Result<[u8; 4], SensorError> {
        // wake up the sensor by pulling the pin down
        self.pin.set_as_output();
        self.pin.set_low();
        self.delay.delay_us(1000);

        // wait for the pin to go up again and then drop to low for 20-40us
        self.pin.set_as_input();
        let _ = wait_for_state(|| self.pin.is_high(), &mut self.delay);
        let _ = wait_for_state(|| self.pin.is_low(), &mut self.delay);

        // another state flip, 80us for both low and high
        let _ = wait_for_state(|| self.pin.is_high(), &mut self.delay);
        let _ = wait_for_state(|| self.pin.is_low(), &mut self.delay);

        // data read starts here
        let mut buf = [42u8; 4];

        for idx in 0..4 {
            buf[idx] = self.read_byte();
        }
        let checksum = self.read_byte();
        if checksum != buf.iter().fold(0, |acc: u8, a: &u8| acc.wrapping_add(*a)) {
            return Err(SensorError::ChecksumMismatch);
        }

        Ok(buf)
    }

    fn read_byte(&mut self) -> u8 {
        let mut buf = 0u8;
        for idx in 0..8 {
            let _ = wait_for_state(|| self.pin.is_high(), &mut self.delay);
            let t = wait_for_state(|| self.pin.is_low(), &mut self.delay);

            if t > 35 {
                buf |= 1 << 7 - idx;
            }
        }
        buf
    }
}
