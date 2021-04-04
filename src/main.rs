
#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use gpio::{GpioOut};
use std::{thread, time};


#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/pump/<pin>/<enable>")]
fn pump(pin: u16, enable: bool) -> &'static str {
    let mut gpio = gpio::sysfs::SysFsGpioOutput::open(pin).unwrap();
    gpio.set_value(enable).expect("could not set gpio4");
    "Complete"
}

fn main() {
    rocket::ignite()
    .mount("/", routes![index])
    .mount("/", routes![pump])
    .launch();
}