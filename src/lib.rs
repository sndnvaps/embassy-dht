/* usage
 *  //for dht22
 * use embassy_executor::Spawner;
 * use defmt::*;
 * use embassy_time::{Delay, Duration, Timer};
 * use embassy_rp;
 * use embassy_dht::dht22::DHT22;
 *
 * #[embassy_executor::main]
 *   async fn main(spawner: Spawner) {
 *  info!("Hello World!");
 *
 *   let p = embassy_rp::init(Default::default());
 *
 *  info!("set up dhtxx pin");
 *
 *   let mut dht_pin = DHT22::new(p.PIN_22,Delay);
 *
 *   loop {
 *   let dht_reading = dht_pin.read().unwrap();
 *   let (temp, humi) = (dht_reading.get_temp(), dht_reading.get_hum());
 *  defmt::info!("Temp = {}, Humi = {}\n", temp,humi);
 *   ... the code what you write
 *}
 *}
 *
 *  //for dht11
 * use embassy_executor::Spawner;
 * use defmt::*;
 * use embassy_time::{Delay, Duration, Timer};
 * use embassy_rp;
 * use embassy_dht::dht11::DHT11;
 *
 * #[embassy_executor::main]
 *  async fn main(spawner: Spawner) {
 *  info!("Hello World!");
 *
 *   let p = embassy_rp::init(Default::default());
 *
 *   info!("set up dhtxx pin");
 *
 *   let mut dht_pin = DHT11::new(p.PIN_22,Delay);
 *
 *  loop {
 *   let dht_reading = dht_pin.read().unwrap();
 *  let (temp, humi) = (dht_reading.get_temp(), dht_reading.get_hum());
 *  defmt::info!("Temp = {}, Humi = {}\n", temp,humi);
 *   ... the code what you write
 *}
 *}
 *
*/

#![no_std]
use embassy_rp::gpio::{Flex, Pin};
use embassy_rp::Peripheral;
use embedded_hal::delay::DelayNs;
use num_traits::float::FloatCore;

#[derive(Debug, Copy, Clone)]
pub struct Reading<T, H> {
    pub temp: T,
    pub hum: H,
}

const WAIT_STEP: u32 = 5;
const MAX_WAIT: u32 = 100;

fn wait_for_state<F, D>(f: F, delay: &mut D) -> u32
where
    F: Fn() -> bool,
    D: DelayNs,
{
    let mut t = 0;
    loop {
        if f() || t > MAX_WAIT {
            return t;
        }
        t += WAIT_STEP;
        delay.delay_us(WAIT_STEP);
    }
}

pub mod dht11 {
    use super::*;

    impl Reading<i8, u8> {
        pub fn get_temp(&self) -> i8 {
            self.temp
        }
        pub fn get_hum(&self) -> u8 {
            self.hum
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
}

pub mod dht22 {
    use super::*;

    impl Reading<f32, f32> {
        pub fn get_temp(&self) -> f32 {
            self.temp
        }
        pub fn get_hum(&self) -> f32 {
            self.hum
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
}
