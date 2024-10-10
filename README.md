# Embassy DHT Sensor Library

  This Rust library provides an interface for interacting with DHT1X and DHT2X temperature and humidity sensors using the Embassy framework.

Only test on raspberry pico (w) (RP2040)

 Pico 2 (RP2350) may be work

# Getting Started

## Installation

Add embassy-dht-sensor to your Cargo.toml:

```toml
[dependencies]
embassy-dht = "0.1.8"
```
## Usage

  Initialize your Raspberry Pi Pico board with Embassy. Create an instance of DHTSensor with the GPIO pin connected to your DHT sensor. Use the read method to get temperature and humidity readings.

## Example

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
embassy-dht = { version = "0.1.7", features= ["embedded_alloc"] }
```
# when enable embedded_alloc we will get new fn in DHT11/DHT22 mod

```rust
pub trait DhtValueString {
     fn get_temp_str(&self) -> String;
     fn get_hum_str(&self) -> String;
}

```

# examples

https://github.com/sndnvaps/embassy-dht/tree/main/examples/picow-display-embedded-alloc

https://github.com/sndnvaps/embassy-dht/tree/main/examples/picow-display

Pick up idea from https://crates.io/crates/embassy-dht-sensor