
#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use rust_gpiozero::*;
use std::thread::sleep;
use std::time::Duration; 



#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/pump")]
fn pump() -> &'static str {

    let pin = LED::new(4);
    pin.on();
    sleep(Duration::from_secs(5));
    pin.off();
    "Pump it up"
}

fn main() {
    rocket::ignite()
    .mount("/", routes![index])
    .mount("/", routes![pump])
    .launch();
}