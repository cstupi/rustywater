#[macro_use] extern crate rocket;

use jwt::JWTAuth;
use settings::Settings;
use gpio::{GpioOut};
use rocket::{State, fairing::AdHoc};
use std::thread::spawn;
use std::thread::sleep;
use std::time::Duration;
use crate::cors::CORSFairing;
mod cors;
mod jwt;
mod settings;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[options("/<_..>")]
fn all_options() {
    /* Intentionally left empty */
}

#[get("/checkauth")]
fn checkauth(_token: JWTAuth<'_>) -> &'static str {
    "Hello, world!"
}


#[put("/<pin>/toggle/<enable>")]
fn toggle(pin: u16, enable: bool, config: &State<Settings>) {
    if !config.gpio_enabled {
        return
    }
    let mut gpio = gpio::sysfs::SysFsGpioOutput::open(pin).unwrap();
    gpio.set_value(enable).expect("could not set gpio4");
}

#[put("/<pin>/blink/<interval>/count/<count>")]
fn blink(pin: u16, interval: u64, count: u64, config: &State<Settings>) {
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
fn timed(pin: u16, time: u64, config: &State<Settings>) {
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
    .mount("/", routes![index, checkauth, all_options])
    .mount("/pin", routes![toggle, timed, blink])
    .attach(AdHoc::config::<Settings>())
    .attach(CORSFairing)
}