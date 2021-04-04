
#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use rust_gpiozero::*;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/pump/<pin>/<enable>")]
fn pump(pin: u8, enable: bool) {
    let pin = LED::new(pin);
    if enable {
        pin.on();
    } else {
        pin.off();
    }
}

fn main() {
    rocket::ignite()
    .mount("/", routes![index])
    .mount("/", routes![pump])
    .launch();
}