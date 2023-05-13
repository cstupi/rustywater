#[macro_use] extern crate rocket;

use jsonwebtoken::jwk;
use jwkstore::JwkStore;
use jwtguard::JwtGuard;
use rocket::Rocket;
use rocket::serde::json;
use rocket::shield::Shield;
use crate::settings::Settings;
use gpio::{GpioOut};
use rocket::{State, fairing::AdHoc};
use std::sync::Arc;
use std::thread::spawn;
use std::thread::sleep;
use std::time::Duration;
use crate::cors::CORSFairing;
mod cors;
mod jwtguard;
mod jwkstore;
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
fn checkauth(_token: JwtGuard<'_>) -> &'static str {
    "Hello, world!"
}


#[put("/<pin>/toggle/<enable>")]
fn toggle(pin: u16, enable: bool, config: &State<Settings>, _token: JwtGuard<'_>) {
    if !config.gpio_enabled {
        return
    }
    let mut gpio = gpio::sysfs::SysFsGpioOutput::open(pin).unwrap();
    gpio.set_value(enable).expect("could not set gpio4");
}

#[put("/<pin>/blink/<interval>/count/<count>")]
fn blink(pin: u16, interval: u64, count: u64, config: &State<Settings>, _token: JwtGuard<'_>) {
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
fn timed(pin: u16, time: u64, config: &State<Settings>, _token: JwtGuard<'_>) {
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
        .attach(Shield::default())
        .attach(AdHoc::config::<Settings>())
        .attach(AdHoc::on_ignite("JwkStore", |rocket| Box::pin(async move {
            let config: Settings = rocket.figment().extract().expect("configuration");
            println!("The jwk url is: {}", config.jwk_url);
            let body = reqwest::get(config.jwk_url).await.expect("failed to call external source for jwkset");
            let text = body.text().await.expect("failed to get jwk request body");
            let jwks: jwk::JwkSet = json::from_str(&text).expect("JWK did not deserialize");
            let jwk_store = JwkStore {
                jwks: jwks.clone(),
            };
            rocket.manage(jwk_store)
        })))
        .attach(CORSFairing)
}