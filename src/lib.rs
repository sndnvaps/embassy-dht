/* usage
 *  //for dht22
 * use embassy_executor::Spawner;
 * use defmt::*;
 * use embassy_time::{Delay, Timer};
 * use embassy_rp;
 * use embassy_dht::dht22::DHT22;
 *
 * #[embassy_executor::main]
 *   async fn main(_spawner: Spawner) {
 *   info!("Hello World!");
 *
 *   let p = embassy_rp::init(Default::default());
 *
 *   info!("set up dhtxx pin");
 *
 *   let mut dht_pin = DHT22::new(p.PIN_22,Delay);
 *
 *   loop {
 *      Timer::after_secs(1).await;
 *      let dht_reading = dht_pin.read().unwrap();
 *      let (temp, humi) = (dht_reading.get_temp(), dht_reading.get_hum());
 *      defmt::info!("Temp = {}, Humi = {}\n", temp,humi);
 *      ... the code what you write
 *   }
 *}
 *
 *  //for dht11
 * use embassy_executor::Spawner;
 * use defmt::*;
 * use embassy_time::{Delay, Timer};
 * use embassy_rp;
 * use embassy_dht::dht11::DHT11;
 *
 * #[embassy_executor::main]
 *  async fn main(_spawner: Spawner) {
 *  info!("Hello World!");
 *
 *  let p = embassy_rp::init(Default::default());
 *
 *  info!("set up dhtxx pin");
 *
 *  let mut dht_pin = DHT11::new(p.PIN_22,Delay);
 *
 *  loop {
 *      Timer::after_secs(1).await;
 *      let dht_reading = dht_pin.read().unwrap();
 *      let (temp, humi) = (dht_reading.get_temp(), dht_reading.get_hum());
 *      defmt::info!("Temp = {}, Humi = {}\n", temp,humi);
 *      ... the code what you write
 *  }
 *}
 *
*/

#![no_std]

use embedded_hal::delay::DelayNs;
use num_traits::float::FloatCore;
#[cfg(feature = "embedded_alloc")]
extern crate alloc;
#[cfg(feature = "embedded_alloc")]
use alloc::{borrow::ToOwned, string::String, string::ToString};

pub mod prelude;

#[cfg(feature = "dht11")]
pub mod dht11;

#[cfg(feature = "dht22")]
pub mod dht22;

#[cfg(feature = "dht20")]
pub mod dht20;

/// Possible errors when interacting with the sensor.
#[derive(Debug)]
pub enum SensorError {
    ChecksumMismatch,
}

#[derive(Debug, Copy, Clone)]
pub struct Reading<T, H> {
    pub temp: T,
    pub hum: H,
}

impl Reading<i8, u8> {
    pub fn get_temp(&self) -> i8 {
        self.temp
    }
    pub fn get_hum(&self) -> u8 {
        self.hum
    }
}

impl Reading<f32, f32> {
    pub fn get_temp(&self) -> f32 {
        self.temp
    }

    pub fn get_hum(&self) -> f32 {
        self.hum
    }
}

#[cfg(feature = "embedded_alloc")]
pub trait DhtValueString {
    fn get_temp_str(&self) -> String;
    fn get_hum_str(&self) -> String;
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

const WAIT_STEP: u32 = 5;
const MAX_WAIT: u32 = 100;

pub(crate) fn wait_for_state<F, D>(f: F, delay: &mut D) -> u32
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
