use embassy_rp::gpio::{Flex, Pin};
use embassy_rp::Peripheral;
use embedded_hal::delay::DelayNs;

#[cfg(feature = "embedded_alloc")]
extern crate alloc;
#[cfg(feature = "embedded_alloc")]
use alloc::borrow::ToOwned;
#[cfg(feature = "embedded_alloc")]
use alloc::string::{String, ToString};

use crate::wait_for_state;
use crate::Reading;

#[cfg(feature = "embedded_alloc")]
use crate::DhtValueString;

impl Reading<i8, u8> {
    pub fn get_temp(&self) -> i8 {
        self.temp
    }
    pub fn get_hum(&self) -> u8 {
        self.hum
    }
}
#[cfg(feature = "embedded_alloc")]
impl DhtValueString for Reading<i8, u8> {
    fn get_temp_str(&self) -> String {
        let temp = self.get_temp();
        let temp_str = temp.to_string();
        temp_str.to_owned()
    }
    fn get_hum_str(&self) -> String {
        let hum = self.get_hum();
        let hum_str = hum.to_string();
        hum_str.to_owned()
    }
}
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
    pub fn read(&mut self) -> Result<Reading<i8, u8>, &str> {
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

    fn read_raw(&mut self) -> Result<[u8; 4], &str> {
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
            return Err("Checksum error");
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
