// Copyright 2024 Developers of the embassy-dht project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! DHT11 functionality
//!

use embassy_rp::gpio::{Flex, Pin};
use embassy_rp::Peripheral;
use embedded_hal::delay::DelayNs;

use crate::wait_for_state;
use crate::{Reading, SensorError};

pub struct DHT11<'a, D> {
    pub pin: Flex<'a>,
    pub delay: D,
}

impl<'a, D> DHT11<'a, D>
where
    D: DelayNs,
{
    pub fn new(pin: impl Peripheral<P = impl Pin> + 'a, delay: D) -> Self {
        let pin = Flex::new(pin);
        Self { pin, delay }
    }

    //pub fn read(&mut self) -> Result<Reading, &str> {
    pub fn read(&mut self) -> Result<Reading<i8, u8>, SensorError> {
        let data = self.read_raw()?;
        let rh = data[0];
        let temp_signed = data[2];
        let temp = {
            let (signed, magnitude) = convert_signed(temp_signed);
            let temp_sign = if signed { -1 } else { 1 };
            temp_sign * magnitude as i8
        };

        Ok(Reading {
            temp: temp,
            hum: rh,
        })
    }

    fn read_raw(&mut self) -> Result<[u8; 4], SensorError> {
        // wake up the sensor by pulling the pin down

        self.pin.set_as_output();
        self.pin.set_low();
        self.delay.delay_ms(18);
        self.pin.set_high();
        self.delay.delay_us(48);

        // wait for the pin to go up again and then drop to low for 20-40us
        self.pin.set_as_input();
        let _ = wait_for_state(|| self.pin.is_high(), &mut self.delay);
        let _ = wait_for_state(|| self.pin.is_low(), &mut self.delay);

        // data read starts here
        let mut buf = [0; 4];

        for idx in 0..4 {
            buf[idx] = self.read_byte();
        }
        let checksum = self.read_byte();
        if checksum
            != buf
                .iter()
                .fold(0_u8, |acc: u8, a: &u8| acc.wrapping_add(*a))
        {
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

fn convert_signed(signed: u8) -> (bool, u8) {
    let sign = signed & 0x80 != 0;
    let magnitude = signed & 0x7F;
    (sign, magnitude)
}
