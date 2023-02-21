#[macro_use] extern crate rocket;

use gpio::{GpioOut};
use jsonwebtoken::jwk::AlgorithmParameters;
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey, decode_header, jwk};
use rocket::http::Status;
use rocket::request::{self, Outcome, Request, FromRequest};
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::{json};
use rocket::{State, fairing::AdHoc};
use std::collections::HashMap;
use std::thread::spawn;
use std::thread::sleep;
use std::time::Duration;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Config {
    gpio_enabled: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Claims {
    sub: String,
    company: String,
    exp: usize,
}

struct ApiKey<'r>(&'r str);

#[derive(Debug)]
enum ApiKeyError {
    Missing,
    Invalid,
}



#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey<'r> {
    type Error = ApiKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        /// Returns true if `key` is a valid API key string.
        fn is_valid(bearerkey: &str) -> bool {

            const JWKS_REPLY: &str = r#"
{"keys":[{"kid":"Itsb713OylqML6v9LxUaaSc9dK_lGfKvaUFPcpCnVsM","kty":"RSA","alg":"RS256","use":"sig","n":"qQC4baQ3MvrYEVytnbXTrRlAEtP1ObNJ-phT0knIraewI5Itoa57rRpt23MftyMVH0UX5SF4fwTnXt3eHq0jEcrL_zP9TAigZ62x-inxxGjmmCxPtWi8afdNOW4W-ATkg0ga0kbD0Ir-0jtVscbZ6yqrrdgTmRcw7cToBf_6Hmg6sf7D6w73o4qJU6R5by8vL12TnhFFOF95_w7ab0GhIWQFiYI8f0v8Rp9XBx-G1PejVuynQMeFVo5rSDYJuFkDWr5PxtyE4y6psvmr68_XrZYSPol4QJhIo8OaNDsfO57zwJPsjGFUaQEY94FWFo6xLhA1cBed7bcDJTa_NT6g9Q","e":"AQAB","x5c":["MIICozCCAYsCBgGGcNEigzANBgkqhkiG9w0BAQsFADAVMRMwEQYDVQQDDApnYXJhZ2UtZGV2MB4XDTIzMDIyMDIxNTEzOVoXDTMzMDIyMDIxNTMxOVowFTETMBEGA1UEAwwKZ2FyYWdlLWRldjCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBAKkAuG2kNzL62BFcrZ21060ZQBLT9TmzSfqYU9JJyK2nsCOSLaGue60abdtzH7cjFR9FF+UheH8E517d3h6tIxHKy/8z/UwIoGetsfop8cRo5pgsT7VovGn3TTluFvgE5INIGtJGw9CK/tI7VbHG2esqq63YE5kXMO3E6AX/+h5oOrH+w+sO96OKiVOkeW8vLy9dk54RRThfef8O2m9BoSFkBYmCPH9L/EafVwcfhtT3o1bsp0DHhVaOa0g2CbhZA1q+T8bchOMuqbL5q+vP162WEj6JeECYSKPDmjQ7Hzue88CT7IxhVGkBGPeBVhaOsS4QNXAXne23AyU2vzU+oPUCAwEAATANBgkqhkiG9w0BAQsFAAOCAQEAWQZcCrlHqQrVwngAlWilN3ok0b/n0cpteionimwC8srk9COia19Y1dKRLzAt/CPcR8BGbz4fcq7yi306USoh+7hita4PFPEQ3KaIJM2N0O/6AhQO3NLVimY/hbzw0sqj1dSNOtpZK3VDAWneqSz2CtRYTrt5mkNEIacAitS8COJH3Yk1IDspXKws7ALEtFMh69B4Bn5Jt36KP1QLRAluHk8/iJUVQYVlTWstFoceIO27u3HAecrj7QOFbAE5hI4zopLwc/mGAr3KjHXKSmEQuL+DN8cmgLyttPs4yRU1j4GV0X9pGgL1Me9KdKaAPb0LwX14qvGU55po/gB12oejvQ=="],"x5t":"G7dG_p55TkXZI8yCJ7nIaWjadaA","x5t#S256":"w3LBuxg969xhmOj0NRYLTKuYOoilnVcEwYVFUF4-FV8"}]}
"#;         let key = bearerkey.replace("Bearer ", "");

            let jwks: jwk::JwkSet = json::from_str(JWKS_REPLY).unwrap();

            let header = match decode_header(&key) {
                Err(e) => return false,
                Ok(t) => t,
            };
            let kid = match header.kid {
                Some(k) => k,
                None => return false,
            };
            if let Some(j) = jwks.find(&kid) {
                match &j.algorithm {
                    AlgorithmParameters::RSA(rsa) => {
                        let decoding_key = DecodingKey::from_rsa_components(&rsa.n, &rsa.e).unwrap();
                        let mut validation = Validation::new(j.common.algorithm.unwrap());
                        validation.validate_exp = false;
                        let decoded_token =
                            decode::<HashMap<String, json::Value>>(&key, &decoding_key, &validation)
                                .unwrap();
                        println!("{:?}", decoded_token.claims);
                    }
                    _ => unreachable!("this should be a RSA"),
                }
            } else {
                return false
            }
            true
        }

        match req.headers().get_one("authorization") {
            None => Outcome::Failure((Status::BadRequest, ApiKeyError::Missing)),
            Some(key) if is_valid(key) => Outcome::Success(ApiKey(key)),
            Some(_) => Outcome::Failure((Status::BadRequest, ApiKeyError::Invalid)),
        }
    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/checkauth")]
fn checkauth(_token: ApiKey<'_>) -> &'static str {
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
    .mount("/", routes![index, checkauth])
    .mount("/pin", routes![toggle, timed, blink])
    .attach(AdHoc::config::<Config>())
}