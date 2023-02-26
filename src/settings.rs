use rocket::serde::{Deserialize};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Settings {
    pub gpio_enabled: bool,
    pub jwk_url: String,
}