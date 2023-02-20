#[macro_use] extern crate rocket;

use gpio::{GpioOut};
use rocket::serde::Deserialize;
use rocket::{State, fairing::AdHoc};
use std::thread::spawn;
use std::thread::sleep;
use std::time::Duration;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Config {
    gpio_enabled: bool,
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[put("/<pin>/toggle/<enable>")]
fn toggle(pin: u16, enable: bool, config: &State<Config>) {
    if !config.gpio_enabled {
        return
    }
    let mut gpio = gpio::sysfs::SysFsGpioOutput::open(pin).unwrap();
    gpio.set_value(enable).expect("could not set gpio4");
}

#[put("/<pin>/blink/<interval>/count/<count>")]
fn blink(pin: u16, interval: u64, count: u64, config: &State<Config>) {
    if !config.gpio_enabled {
        return
    }
    spawn(move || {
        let mut gpio = gpio::sysfs::SysFsGpioOutput::open(pin).unwrap();
        for _i in 1..count {
            gpio.set_value(true).expect("could not set pin");
            sleep(Duration::from_secs(interval));
            gpio.set_value(false).expect("could not set pin");
        }
    });
}

// Webhook compatible way to turn on a pin for a time
#[get("/<pin>/timed/<time>")]
fn timed(pin: u16, time: u64, config: &State<Config>) {
    println!("GPIO ENABLED: {}", config.gpio_enabled);
    if !config.gpio_enabled {
        return
    }
    spawn(move || {
        let mut gpio = gpio::sysfs::SysFsGpioOutput::open(pin).unwrap();
        gpio.set_value(true).expect("could not set pin");
        sleep(Duration::from_secs(time));
        gpio.set_value(false).expect("could not set pin");
        println!("Pin: {}, Time: {}", pin, time);
    });
}

#[launch]
fn rocket() -> _ {
    rocket::build()
    .mount("/", routes![index])
    .mount("/pin", routes![toggle, timed, blink])
    .attach(AdHoc::config::<Config>())
}