    use dht11/dht22 to get the temp & humi then show it by ssd1306 oled screen

# ssd1306 use I2C1 interface

    SDA -> GP2 (On Board pin 4)
    SCL -> GP3 (On Board pin 5)

# DHT11/DHT22 connect to GP22(On Board pin 29)

# Example picow-display default feature

 `Modified` Cargo.toml to change default support feature
```toml
[features]
# if you want make dht11/dht22 as default feature, just change the comment line
# default = ["dht11"]
default = ["dht22"]
dht11 = []
dht22 = []
```

# Deploy by elf2uf2-rs

    cargo install elf2uf2-rs 