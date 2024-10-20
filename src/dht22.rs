use embassy_rp::gpio::{Flex, Pin};
use embassy_rp::Peripheral;
use embedded_hal::delay::DelayNs;
use num_traits::float::FloatCore;
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

impl Reading<f32, f32> {
    pub fn get_temp(&self) -> f32 {
        self.temp
    }

    pub fn get_hum(&self) -> f32 {
        self.hum
    }
}

#[cfg(feature = "embedded_alloc")]
impl DhtValueString for Reading<f32, f32> {
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

//rust f32 custom decimal point length pick up from
//https://zhuanlan.zhihu.com/p/466389032
trait F32Utils {
    fn round_fixed(self, n: u32) -> f32;
}

impl F32Utils for f32 {
    fn round_fixed(self, n: u32) -> f32 {
        if n <= 0 {
            return self.round();
        }
        let i = 10_usize.pow(n) as f32;
        let x = self * i;
        if self > 0_f32 {
            // 正数情况下 1.15_f32.round() 为1.2
            let m = x.round() as u32;
            m as f32 / i
        } else {
            //默认的负数round四舍五入取整(a) -1.15_f32.round() 为 -1.2 (b)
            let mr = x.trunc(); //整数部分
            let mf = x.fract(); //小数部分
            if mf.abs() >= 0.5 {
                // -3.14159 四舍五入保留3位 则-3141.59 / 1000 = -3.14159(逢五进一) 变为-3.140
                return (mr + 1_f32) / i;
            }
            //小数部分 < 0.5直接舍弃小数部分；小数点直接使用整数部分向前移动n位
            mr / i
        }
    }
}

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

    pub fn read(&mut self) -> Result<Reading<f32, f32>, &str> {
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

    fn read_raw(&mut self) -> Result<[u8; 4], &str> {
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
