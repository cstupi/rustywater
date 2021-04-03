use rust_gpiozero::*;
use std::thread::sleep;
use std::time::Duration; 

fn main() {
    let pin = LED::new(4);
    pin.on();
    sleep(Duration::from_secs(5));
    pin.off();
    println!("Hello, world!");
}
