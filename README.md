# Embassy DHT Sensor Library

  This Rust library provides an interface for interacting with DHT1X and DHT2X temperature and humidity sensors using the Embassy framework.

 test on raspberry pico (w) (RP2040) &&  Pico 2 (RP2350)

# Getting Started

## Installation

Add embassy-dht-sensor to your Cargo.toml:

```toml
[dependencies]
embassy-dht = "0.1.9"
```
## Usage

  Initialize your Raspberry Pi Pico board with Embassy. Create an instance of DHTSensor with the GPIO pin connected to your DHT sensor. Use the read method to get temperature and humidity readings.

## Example for RP2040

```rust
  //   for dht22
  use embassy_executor::Spawner;
  use defmt::*;
  use embassy_time::{Delay, Timer};
  use embassy_rp;
  use embassy_dht::dht22::DHT22;
 
  #[embassy_executor::main]
    async fn main(spawner: Spawner) {
    info!("Hello World!");
 
    let p = embassy_rp::init(Default::default());
 
    info!("set up dhtxx pin");
 
    let mut dht_pin = DHT22::new(p.PIN_22,Delay);
 
    loop {
    Timer::after_secs(1).await;
    let dht_reading = dht_pin.read().unwrap();
    let (temp, humi) = (dht_reading.get_temp(), dht_reading.get_hum());
    defmt::info!("Temp = {}, Humi = {}\n", temp,humi);
    ... the code what you write
 }
 }
```
```rust
  // for dht11
  use embassy_executor::Spawner;
  use defmt::*;
  use embassy_time::{Delay, Timer};
  use embassy_rp;
  use embassy_dht::dht11::DHT11;
 
  #[embassy_executor::main]
    async fn main(spawner: Spawner) {
    info!("Hello World!");
 
    let p = embassy_rp::init(Default::default());
 
    info!("set up dhtxx pin");
 
    let mut dht_pin = DHT11::new(p.PIN_22,Delay);
 
   loop {
    Timer::after_secs(1).await;
    let dht_reading = dht_pin.read().unwrap();
    let (temp, humi) = (dht_reading.get_temp(), dht_reading.get_hum());
    defmt::info!("Temp = {}, Humi = {}\n", temp,humi);
    ... the code what you write
   }
   }
```

# New feature embedded_alloc

to enable it by ,add to Cargo.toml

```toml
embassy-dht = { version = "0.1.9", features= ["embedded_alloc"] }
```
# when enable embedded_alloc we will get new fn in DHT11/DHT22 mod

```rust
pub trait DhtValueString {
     fn get_temp_str(&self) -> String;
     fn get_hum_str(&self) -> String;
}

```

## Example for RP2350

  for rp2350 need to use the crate embassy-rp from github.com 

  use rev="ee669ee5c57851ade034beca7cfaf81825c4c21b"

Cargo.toml

```toml
embassy-executor = { version = "0.6.0", git="https://github.com/embassy-rs/embassy", rev="ee669ee5c57851ade034beca7cfaf81825c4c21b", features = ["task-arena-size-98304", "arch-cortex-m", "executor-thread", "executor-interrupt", "defmt", "integrated-timers"] }
embassy-time = { version = "0.3.2", git="https://github.com/embassy-rs/embassy", rev="ee669ee5c57851ade034beca7cfaf81825c4c21b",features = ["defmt", "defmt-timestamp-uptime"] }
embassy-time-driver = { version = "0.1", git="https://github.com/embassy-rs/embassy", rev="ee669ee5c57851ade034beca7cfaf81825c4c21b"}
embassy-rp = { version = "0.2.0", git="https://github.com/embassy-rs/embassy", rev="ee669ee5c57851ade034beca7cfaf81825c4c21b", features = ["defmt", "unstable-pac", "time-driver", "critical-section-impl","rp235xa", "binary-info"] }
embassy-dht = { version = "0.1.9", features = [ "embedded_alloc"] }
...what crate you need
```

src/main.rs
```rust
#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::block::ImageDef;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::i2c;
use embassy_time::{Delay, Timer};

use {defmt_rtt as _, panic_probe as _};

use embassy_dht::dht22::DHT22;

#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {

    let p = embassy_rp::init(Default::default());

    info!("set up dhtxx pin");

    let mut dht_pin = DHT22::new(p.PIN_17, Delay);

    //enable on board LED
    let mut led = Output::new(p.PIN_25, Level::Low);


    loop {
        Timer::after_secs(1).await;

        let dht_reading = dht_pin.read().unwrap();

        // Perform a sensor reading
        let (temp, humi) = (dht_reading.get_temp(), dht_reading.get_hum());

       ... what you code 

        info!("led on!");
        led.set_high();
        Timer::after_millis(250).await;

        info!("led off!");
        led.set_low();
        Timer::after_millis(250).await;
    }
}
```

# examples

https://github.com/sndnvaps/embassy-dht/tree/main/examples/picow-display-embedded-alloc

https://github.com/sndnvaps/embassy-dht/tree/main/examples/picow-display

https://github.com/sndnvaps/embassy-dht/tree/main/examples/pico2-display


Pick up idea from https://crates.io/crates/embassy-dht-sensor