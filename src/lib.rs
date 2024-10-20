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
#[cfg(feature = "embedded_alloc")]
extern crate alloc;
#[cfg(feature = "embedded_alloc")]
use alloc::string::String;

#[cfg(feature = "dht11")]
pub mod dht11;

#[cfg(feature = "dht22")]
pub mod dht22;

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

#[cfg(feature = "embedded_alloc")]
pub trait DhtValueString {
    fn get_temp_str(&self) -> String;
    fn get_hum_str(&self) -> String;
}
